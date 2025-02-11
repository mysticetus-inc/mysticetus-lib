#![feature(
    const_try,
    doc_cfg,
    step_trait,
    const_trait_impl,
    pattern,
    int_roundings,
    let_chains
)]
#![deny(clippy::suspicious, clippy::complexity, clippy::perf, clippy::style)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, cfg(feature = "protos"))]
//! A [`Timestamp`] definition that supports conversions with different timestamp/datetime
//! objects from common crates.
//!
//! Supported types are:
//!
//! - [`time::OffsetDateTime`]
//! - [`chrono::DateTime`]
//! - [`std::time::SystemTime`]
//! - [`protos::protobuf::Timestamp`] (which is what a [`Timestamp`] is internally)
//!
//! Computing offsets with `Duration` types are also supported by both [`std::time::Duration`]
//! and [`time::Duration`].
//!
//! [`time::OffsetDateTime`]: [`::time::OffsetDateTime`]
//! [`time::Duration`]: [`::time::Duration`]
use std::fmt;

pub mod date;
pub mod de;
pub mod duration;
pub mod error;
mod macros;
mod month;
pub mod nanos;
mod parse;
mod ser;
pub mod time;
pub mod timed;
mod timestamp;
pub(crate) mod util;

pub use crate::date::Date;
pub use crate::duration::Duration;
pub use crate::error::Error;
pub use crate::month::Month;
pub use crate::time::Time;
pub use crate::timed::Timed;
pub use crate::timestamp::Timestamp;

/// Enum with the Unit of time used in this crate. Used throughout the crate to specify errors,
/// expected Unit, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Units of seconds.
    Seconds,
    /// Units of milliseconds
    Millis,
    /// Units of microseconds,
    Micros,
    /// Units of nanoseconds.
    Nanos,
}

impl Unit {
    /// Returns a `&'static [`str`]` with the name of the unit for formatting
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Seconds => "seconds",
            Self::Millis => "milliseconds",
            Self::Micros => "microseconds",
            Self::Nanos => "nanoseconds",
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

/// Conversion constants between Unit of time.
pub(crate) mod conv {
    /// Number of nanoseconds per second.
    pub(crate) const NANOS_PER_SECOND_I32: i32 = 1_000_000_000;
    pub(crate) const NANOS_PER_SECOND_I64: i64 = 1_000_000_000;
    pub(crate) const NANOS_PER_SECOND_I128: i128 = 1_000_000_000;

    /// Number of nanoseconds per second, as an [`f64`].
    pub(crate) const NANOS_PER_SECOND_F64: f64 = 1e9;

    /// Number of milliseconds per second.
    pub(crate) const MILLIS_PER_SECOND: i64 = 1_000;

    /// Number of milliseconds per second, in [`f64`] form.
    pub(crate) const MILLIS_PER_SECOND_F64: f64 = 1e3;

    /// Number of milliseconds per second
    pub(crate) const MICROS_PER_SECOND: i64 = 1_000_000;

    /// Number of nanoseconds per millisecond.
    pub(crate) const NANOS_PER_MILLI: i64 = NANOS_PER_SECOND_I64 / MILLIS_PER_SECOND;

    /// Number of nanoseconds per millisecond, in [`f64`].
    pub(crate) const NANOS_PER_MILLI_F64: f64 = NANOS_PER_SECOND_F64 / MILLIS_PER_SECOND_F64;

    /// Number of nanoseconds per Microsoft Tick.
    pub(crate) const NANOS_PER_TICK: i64 = 100;

    /// Offset for converting ticks -> unix epoch seconds. Pulled directly from
    /// MS C# [`DateTimeOffset.cs`] source code.
    ///
    /// [`DateTimeOffset.cs`]: <https://referencesource.microsoft.com/#mscorlib/system/datetimeoffset.cs,7c6f98bb552ffed1>
    pub(crate) const TICKS_UNIX_EPOCH_OFFSET_SECONDS: i64 = 62_135_596_800;

    /// Offset for converting ticks -> unix epoch (in nanoseconds).
    pub(crate) const TICKS_UNIX_EPOCH_OFFSET_NANOS: i128 =
        TICKS_UNIX_EPOCH_OFFSET_SECONDS as i128 * NANOS_PER_SECOND_I128;
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use rand::Rng;

    use super::*;

    #[test]
    fn test_chrono_dt_conversion() {
        let dt = Utc::now();
        let converted_dt = std::hint::black_box(Timestamp::from_datetime(dt).as_datetime());

        assert_eq!(dt, converted_dt);
    }

    #[test]
    fn test_zero_point() {
        let epoch = Utc.timestamp_opt(0, 0).unwrap();
        let dt = Timestamp::from_nanos(0).as_datetime();

        assert_eq!(epoch, dt);
    }

    #[test]
    fn test_add_duration() {
        let rand_ts: i64 = rand::thread_rng().gen_range(1500000000..1650000000);

        let dt = Timestamp::from_seconds(rand_ts);

        let tomorrow_dt = Timestamp::from_seconds(rand_ts + 3600 * 24);

        let computed_tomorrow_dt = dt + ::time::Duration::DAY;

        assert_eq!(tomorrow_dt, computed_tomorrow_dt);
    }
}
