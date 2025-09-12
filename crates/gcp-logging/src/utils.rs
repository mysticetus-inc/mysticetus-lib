use std::backtrace::{Backtrace, BacktraceStatus};
use std::error::Error as StdError;
use std::{fmt, io};

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::options::TryGetBacktrace;

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct SerializeErrorReprs<'a> {
    pub(crate) error: &'a (dyn StdError + 'static),
    pub(crate) source_depth: u8,
    pub(crate) try_get_bt: TryGetBacktrace,
}

const DEFAULT_MAX_DEPTH: u8 = 32;

impl<'a> SerializeErrorReprs<'a> {
    pub(crate) fn new(error: &'a (dyn StdError + 'static), try_get_bt: TryGetBacktrace) -> Self {
        Self {
            error,
            source_depth: 0,
            try_get_bt,
        }
    }

    fn capture_backtrace(&self) -> Option<Backtrace> {
        let bt = match self.try_get_bt {
            TryGetBacktrace::No => return None,
            TryGetBacktrace::Yes => Backtrace::capture(),
            TryGetBacktrace::Force => Backtrace::force_capture(),
        };

        // Only return Some if we actually captured the backtrace.
        if bt.status() == BacktraceStatus::Captured {
            Some(bt)
        } else {
            None
        }
    }

    fn as_serialize_debug_display(&self) -> impl serde::Serialize + '_ {
        struct SerializeDebugDisplayMap<'a, 'b>(&'b SerializeErrorReprs<'a>);

        impl serde::Serialize for SerializeDebugDisplayMap<'_, '_> {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut map = serializer.serialize_map(Some(2))?;
                self.0.serialize_debug_display(&mut map)?;
                map.end()
            }
        }

        SerializeDebugDisplayMap(self)
    }

    fn serialize_debug_display<M>(&self, map: &mut M) -> Result<(), M::Error>
    where
        M: SerializeMap + ?Sized,
    {
        map.serialize_entry("debug", &format_args!("{:?}", self.error))?;
        map.serialize_entry("display", &format_args!("{}", self.error))?;
        Ok(())
    }

    fn serialize_root<M>(&self, map: &mut M) -> Result<(), M::Error>
    where
        M: SerializeMap,
    {
        self.serialize_debug_display(map)?;

        if let Some(bt) = self.capture_backtrace() {
            map.serialize_entry("backtrace", &SerializeBacktrace(&bt))?;
        }

        Ok(())
    }

    fn serialize_no_sources<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bt = match self.capture_backtrace() {
            Some(bt) => bt,
            None => return self.as_serialize_debug_display().serialize(serializer),
        };

        let mut map = serializer.serialize_map(Some(3))?;
        self.serialize_debug_display(&mut map)?;
        map.serialize_entry("backtrace", &SerializeBacktrace(&bt))?;
        map.end()
    }

    fn sources(&self) -> Sources<'a> {
        Sources {
            next: self.source(),
        }
    }

    fn source(&self) -> Option<SerializeErrorReprs<'a>> {
        // check that we aren't too far recursed
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

#[derive(Clone)]
struct Sources<'b> {
    next: Option<SerializeErrorReprs<'b>>,
}

impl serde::Serialize for Sources<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(None)?;

        for source in (Self { next: self.next }) {
            seq.serialize_element(&source.as_serialize_debug_display())?;
        }

        seq.end()
    }
}

impl<'b> Iterator for Sources<'b> {
    type Item = SerializeErrorReprs<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        self.next = current.source();
        Some(current)
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
        let mut sources = self.sources();

        let Some(parent) = sources.next() else {
            return self.serialize_no_sources(serializer);
        };

        let bt = self.capture_backtrace();

        let mut map = serializer.serialize_map(Some(3 + bt.is_some() as usize))?;

        self.serialize_debug_display(&mut map)?;

        if let Some(bt) = bt {
            map.serialize_entry("backtrace", &SerializeBacktrace(&bt))?;
        }

        // try and flatten 'sources' to 'source' if we only have one
        match sources.next() {
            Some(_grandparent) => {
                // get a new iterator, since it avoids needing to make another type to
                // serialize a partially exhausted iter
                map.serialize_entry("sources", &self.sources())?;
            }
            None => map.serialize_entry("source", &parent.as_serialize_debug_display())?,
        }

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

pub(crate) fn with_buffer<O>(with_buf_fn: impl FnOnce(&mut Vec<u8>) -> O) -> O {
    use std::cell::RefCell;

    thread_local! {
        static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024));
    }

    // Need to wrap the callback in an option so we don't have to move it into the
    // closure. If we did, there's no way to call with the fallback buffer it if
    // the TLS value is inaccessible for whatever reason.
    let mut callback = Some(with_buf_fn);

    let res = BUF.try_with(|buf| {
        if let Ok(mut buf) = buf.try_borrow_mut() {
            Some((callback.take().expect("this is Some"))(&mut *buf))
        } else {
            None
        }
    });

    match res {
        Ok(Some(output)) => output,
        Err(_) | Ok(None) => {
            let mut tmp_buf = Vec::with_capacity(512);
            (callback.take().expect("this wasn't removed in the closure"))(&mut tmp_buf)
        }
    }
}

#[derive(Default, Debug)]
pub struct IdHasher(u64);

impl std::hash::Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("SpanId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Newtype to ensure the Hash impl only calls write_u64
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct SpanId(pub tracing::span::Id);

impl From<tracing::span::Id> for SpanId {
    fn from(value: tracing::span::Id) -> Self {
        Self(value)
    }
}

impl std::hash::Hash for SpanId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.into_u64());
    }
}

/// Used to pass data through tracing::Visit impls by hijacking
/// error downcasting.
#[derive(Debug)]
pub struct ErrorPassthrough<T>(pub T);

impl<T> std::fmt::Display for ErrorPassthrough<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorPassthrough({})", std::any::type_name::<T>())
    }
}

impl<T: std::fmt::Debug> std::error::Error for ErrorPassthrough<T> {}

impl<T: std::fmt::Debug + 'static> ErrorPassthrough<T> {
    pub fn as_dyn(&self) -> &(dyn std::error::Error + 'static) {
        self as &(dyn std::error::Error + 'static)
    }

    pub fn try_cast_from<'t>(error: &'t (dyn std::error::Error + 'static)) -> Option<&'t T> {
        error
            .downcast_ref::<Self>()
            .map(|ErrorPassthrough(value)| value)
    }
}

#[cfg(feature = "valuable")]
impl<T: std::fmt::Debug + 'static> valuable::Valuable for ErrorPassthrough<T> {
    fn visit(&self, visit: &mut dyn valuable::Visit) {
        visit.visit_value(self.as_value());
    }

    fn as_value(&self) -> valuable::Value<'_> {
        valuable::Value::Error(self.as_dyn())
    }
}

pub(crate) struct ErrorPassthroughVisitor<F, T, Out> {
    f: F,
    out: Option<Out>,
    _marker: std::marker::PhantomData<T>,
}

impl<F, T, Out> ErrorPassthroughVisitor<F, T, Out> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            out: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn take_output(&mut self) -> Option<Out> {
        self.out.take()
    }

    pub fn into_output(self) -> Option<Out> {
        self.out
    }
}

#[cfg(feature = "valuable")]
impl<F, T, Out> valuable::Visit for ErrorPassthroughVisitor<F, T, Out>
where
    for<'a> F: FnMut(&'a T) -> Out,
    ErrorPassthrough<T>: std::error::Error,
    T: 'static,
{
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if let Some(ErrorPassthrough(value)) = value
            .as_error()
            .and_then(|err| err.downcast_ref::<ErrorPassthrough<T>>())
        {
            self.out = Some((self.f)(value));
        }
    }
}

impl<F, T, Out> tracing::field::Visit for ErrorPassthroughVisitor<F, T, Out>
where
    for<'a> F: FnMut(&'a tracing_core::Field, &'a T) -> Out,
    ErrorPassthrough<T>: std::error::Error,
    T: 'static,
{
    fn record_debug(&mut self, _: &tracing_core::Field, _: &dyn fmt::Debug) {}

    fn record_error(
        &mut self,
        field: &tracing_core::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        if let Some(ErrorPassthrough(value)) = value.downcast_ref::<ErrorPassthrough<T>>() {
            self.out = Some((self.f)(field, value));
        }
    }

    #[cfg(feature = "valuable")]
    fn record_value(&mut self, field: &tracing_core::Field, value: valuable::Value<'_>) {
        if let Some(ErrorPassthrough(value)) = value
            .as_error()
            .and_then(|err| err.downcast_ref::<ErrorPassthrough<T>>())
        {
            self.out = Some((self.f)(field, value));
        }
    }
}

pub struct ErrorVisitor<F>(pub F);

impl<F> tracing::field::Visit for ErrorVisitor<F>
where
    for<'a> F: FnMut(&'a tracing_core::Field, &'a (dyn std::error::Error + 'static)),
{
    fn record_debug(&mut self, _: &tracing_core::Field, _: &dyn fmt::Debug) {}

    fn record_error(
        &mut self,
        field: &tracing_core::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        (self.0)(field, value);
    }

    #[cfg(feature = "valuable")]
    fn record_value(&mut self, field: &tracing_core::Field, value: valuable::Value<'_>) {
        if let Some(error) = value.as_error() {
            (self.0)(field, error);
        }
    }
}

pub(crate) struct HexBytes<'a>(pub(crate) &'a [u8]);

impl fmt::Debug for HexBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter().copied() {
            f.write_fmt(format_args!("{byte:02x}"))?;
        }

        Ok(())
    }
}
