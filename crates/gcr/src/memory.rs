//! Interface to track memory usage/limits.

use std::fmt::Write;
use std::future::poll_fn;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fmt, io};

const USAGE_FILE: &str = "/sys/fs/cgroup/memory/memory.usage_in_bytes";
const LIMIT_FILE: &str = "/sys/fs/cgroup/memory/memory.limit_in_bytes";

/// The current memory limit, in bytes. 0 indicates it's never been checked.
static LIMIT: AtomicU64 = AtomicU64::new(0);

/// The current memory usage, in bytes. 0 indicates it's never been checked.
static CURRENT: AtomicU64 = AtomicU64::new(0);

/// Maximum number of digits in a 64 bit number, plus 1. Used as a buffer size
/// when reading from the usage/limit files above. Cloud run doesn't support
/// memory sizes anywhere even remotely close to u64::MAX bytes, so this should
/// be plenty big.
const MAX_DIGITS: usize = (u64::MAX.ilog10() + 1) as usize;

/// Formatting wrapper around a [`u64`]. in the [`fmt::Display`] impl, this is formatted in
/// user-friendly units of b, KiB, MiB or GiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FmtBytes(pub u64);

impl FmtBytes {
    pub fn with_pair<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&str, &str) -> O,
    {
        let mut b = self.0;
        let mut divs = 0;
        while b > 1024 {
            b /= 1024;
            divs += 1;
        }

        let (num, suffix) = match divs {
            1 => (b, "KiB"),
            2 => (b, "MiB"),
            3 => (b, "GiB"),
            _ => (self.0, "b"),
        };

        let mut buf = itoa::Buffer::new();

        f(buf.format(num), suffix)
    }

    #[allow(dead_code)]
    pub fn format_into_string(&self, s: &mut String) {
        self.with_pair(|num, unit| {
            s.reserve(num.len() + unit.len() + 1);
            s.push_str(num);

            // less clear, yes, but it's also 25% more generated assembly since
            // it has to encode the u32 char rather than just push the 1 byte ascii str.
            #[allow(clippy::single_char_add_str)]
            s.push_str(" ");

            s.push_str(unit);
        });
    }

    pub fn format_into<F: Write>(&self, dst: &mut F) -> fmt::Result {
        self.with_pair(|num, unit| {
            dst.write_str(num)?;

            // less clear, yes, but it's also 25% more generated assembly since
            // it has to encode the u32 char rather than just push the 1 byte ascii str.
            #[allow(clippy::single_char_add_str)]
            dst.write_str(" ")?;

            dst.write_str(unit)
        })
    }
}

impl fmt::Display for FmtBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_into(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryUsage {
    current: u64,
    limit: u64,
}

impl fmt::Display for MemoryUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let percen = self.ratio() * 100.0;
        f.write_str("using ")?;
        f.write_fmt(format_args!("{percen:.1}"))?;
        f.write_str("% of available memory (")?;
        FmtBytes(self.current).format_into(f)?;
        f.write_str(" of ")?;
        FmtBytes(self.limit).format_into(f)?;
        f.write_str(")")
    }
}

impl MemoryUsage {
    pub async fn get() -> io::Result<MemoryUsage> {
        let (current, limit) =
            tokio::try_join!(read_memory_file(USAGE_FILE), read_memory_file(LIMIT_FILE))?;
        LIMIT.store(limit, Ordering::Relaxed);
        CURRENT.store(current, Ordering::Relaxed);

        Ok(MemoryUsage { current, limit })
    }

    pub fn get_last() -> Option<MemoryUsage> {
        macro_rules! return_none_if_0 {
            ($n:expr) => {{
                match $n {
                    0 => return None,
                    non_zero => non_zero,
                }
            }};
        }

        let current = return_none_if_0!(CURRENT.load(Ordering::Relaxed));
        let limit = return_none_if_0!(LIMIT.load(Ordering::Relaxed));

        Some(MemoryUsage { limit, current })
    }

    #[inline]
    pub const fn current(&self) -> u64 {
        self.current
    }

    #[inline]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    #[inline]
    pub fn ratio(&self) -> f64 {
        self.current as f64 / self.limit as f64
    }
}

async fn read_memory_file(path: &str) -> io::Result<u64> {
    use tokio::io::{AsyncRead, ReadBuf};

    let mut file = tokio::fs::File::open(&path).await?;

    let mut dst = MaybeUninit::uninit_array::<MAX_DIGITS>();
    let mut read_buf = ReadBuf::uninit(&mut dst);

    poll_fn(|ctx| Pin::new(&mut file).poll_read(ctx, &mut read_buf)).await?;

    let bytes = read_buf.filled();

    std::str::from_utf8(bytes)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
        .trim() // need to trim, these files will have whitespace at the end.
        .parse::<u64>()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}
