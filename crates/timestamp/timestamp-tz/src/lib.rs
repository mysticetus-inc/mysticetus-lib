#![feature(const_trait_impl)]

use std::fmt;

pub struct TimeZoneInfo {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownTimeZone(String);

impl fmt::Display for UnknownTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown time zone '{}'", self.0)
    }
}

pub type BaseSpan<'a> = Span<'a, ()>;

#[allow(unused)]
pub struct Span<'a, S = i64> {
    name: &'a str,
    utc_offset: i64,
    dst_offset: i64,
    starts: S,
}

mod private {
    use super::Span;

    #[const_trait]
    pub trait Sealed {
        fn new() -> Self
        where
            Self: Sized;

        fn base_span(&self) -> &'static Span<'static, ()>;

        fn remaining_spans(&self) -> &'static [Span<'static, i64>];
    }
}
