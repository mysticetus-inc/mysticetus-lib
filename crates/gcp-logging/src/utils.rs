use std::error::Error as StdError;
use std::{fmt, io};

use serde::Serializer;
use serde::ser::SerializeMap;

use crate::TryGetBacktrace;

pub(crate) struct SerializeDebug<'a, F: ?Sized>(pub &'a F);

impl<F> serde::Serialize for SerializeDebug<'_, F>
where
    F: fmt::Debug + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TlsStringBuf::with_buf(|buf| {
            std::fmt::write(buf, format_args!("{:?}", self.0))
                .expect("string formatting should never fail");

            serializer.serialize_str(buf)
        })
    }
}

pub(crate) struct SerializeDisplay<'a, F: ?Sized>(pub &'a F);

impl<F> serde::Serialize for SerializeDisplay<'_, F>
where
    F: fmt::Display + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TlsStringBuf::with_buf(|buf| {
            std::fmt::write(buf, format_args!("{}", self.0))
                .expect("string formatting should never fail");

            serializer.serialize_str(buf)
        })
    }
}

pub(crate) struct SerializeErrorReprs<'a> {
    pub(crate) error: &'a (dyn StdError + 'static),
    pub(crate) source_depth: usize,
    pub(crate) try_get_bt: crate::TryGetBacktrace,
}

const DEFAULT_MAX_DEPTH: usize = 10;

impl<'a> SerializeErrorReprs<'a> {
    pub(crate) fn new(error: &'a (dyn StdError + 'static), try_get_bt: TryGetBacktrace) -> Self {
        Self {
            error,
            source_depth: 0,
            try_get_bt,
        }
    }

    fn source(&self) -> Option<SerializeErrorReprs<'_>> {
        // check that we arent too far recursed
        if self.source_depth >= DEFAULT_MAX_DEPTH {
            return None;
        }

        self.error.source().map(|error| SerializeErrorReprs {
            error,
            source_depth: self.source_depth + 1,
            try_get_bt: TryGetBacktrace::No,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct JsonFloat(pub f64);

impl serde::Serialize for JsonFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::num::FpCategory::*;

        match self.0.classify() {
            Subnormal | Normal | Zero => serializer.serialize_f64(self.0),
            Infinite if self.0.is_sign_negative() => serializer.serialize_str("-Inf"),
            Infinite => serializer.serialize_str("Inf"),
            Nan => serializer.serialize_str("NaN"),
        }
    }
}

impl serde::Serialize for SerializeErrorReprs<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let source = self.source();

        macro_rules! serialize_debug_display_source {
            ($map:expr) => {{
                $map.serialize_entry("debug", &SerializeDebug(self.error))?;
                $map.serialize_entry("display", &SerializeDisplay(self.error))?;

                if let Some(ref source) = source {
                    $map.serialize_entry("source", source)?;
                }
            }};
        }

        macro_rules! serialize_only_debug_display {
            () => {{
                let mut map = serializer.serialize_map(Some(2 + source.is_some() as usize))?;
                serialize_debug_display_source!(map);
                map.end()
            }};
        }

        let bt = match self.try_get_bt {
            crate::TryGetBacktrace::Force => std::backtrace::Backtrace::force_capture(),
            crate::TryGetBacktrace::Yes => std::backtrace::Backtrace::capture(),
            crate::TryGetBacktrace::No => return serialize_only_debug_display!(),
        };

        // if a backtrace wasn't captured, ignore
        if !matches!(bt.status(), std::backtrace::BacktraceStatus::Captured) {
            return serialize_only_debug_display!();
        }

        let mut map = serializer.serialize_map(Some(3 + source.is_some() as usize))?;

        serialize_debug_display_source!(map);

        map.serialize_entry("backtrace", &SerializeBacktrace(&bt))?;

        map.end()
    }
}

struct SerializeBacktrace<'a>(&'a std::backtrace::Backtrace);

impl serde::Serialize for SerializeBacktrace<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.0)
    }
}

macro_rules! tls_buf {
    ($v:vis $name:ident: $count:literal => $inner_ty:ident :: $($t:tt)*) => {
        $v enum $name { }

        impl $name {
            pub fn with_buf<O>(mut with_fn: impl FnOnce(&mut $inner_ty) -> O) -> O {
                macro_rules! make {
                    (@BUF) => {{
                        $inner_ty::$($t)*
                    }};
                    (@TLS) => {{
                        thread_local! {
                            static TLS: ::std::cell::RefCell<$inner_ty> = ::std::cell::RefCell::new(make!(@BUF));
                        }
                        &TLS
                    }};
                }

                static TLS_BUFS: [&::std::thread::LocalKey<::std::cell::RefCell<$inner_ty>>; $count] = [make!(@TLS); $count];


                for tls_buf in TLS_BUFS.iter() {
                    match tls_buf.with(move |b| {
                        match b.try_borrow_mut() {
                            Ok(mut ref_mut) => {
                                ref_mut.clear();
                                Ok(with_fn(&mut *ref_mut))
                            },
                            Err(_) => Err(with_fn),
                        }
                    })
                    {
                        Ok(ret) => return ret,
                        Err(f) => with_fn = f,
                    }
                }

                let mut buf = make!(@BUF);
                with_fn(&mut buf)
            }
        }
    };
}

tls_buf!(pub TlsStringBuf: 4 => String::with_capacity(256));

pub(crate) struct IoAdapter<F>(pub F);

impl<F> std::io::Write for IoAdapter<F>
where
    F: std::fmt::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.0
            .write_str(s)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(s.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
