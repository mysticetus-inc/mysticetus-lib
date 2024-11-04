//! [`Time`] definition and associated impls.
use std::cmp::Ordering;
use std::{fmt, ops};

use serde::{Deserialize, Serialize};

/// A thin wrapper around [`time::Time`].
#[allow(clippy::derived_hash_with_manual_eq)]
// `PartialEq` has expected behavior, and is only manually implemented for 'const'.
#[derive(Debug, Clone, Copy, Hash)]
pub struct Time(time::Time);

#[cfg(feature = "deepsize")]
deepsize::known_deep_size!(0; Time);

/// Builds a [`Time`] in a const context, checking for out of range values.
#[macro_export]
macro_rules! time {
    ($hrs:literal : $mins:literal) => {
        $crate::time::time!($hrs:$mins:0);
    };
    ($hrs:literal : $mins:literal: $seconds:literal) => {{
        $crate::time::TimeBuilder::new().hours($hrs).minutes($mins).seconds($seconds)
    }};
}

/// A const-capable builder for a [`Time`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeBuilder<H, M> {
    hours: H,
    mins: M,
}

impl Default for TimeBuilder<(), ()> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TimeBuilder<(), ()> {
    /// initializes a new [`TimeBuilder`]
    #[inline]
    pub const fn new() -> Self {
        Self {
            hours: (),
            mins: (),
        }
    }

    /// Insert the hour of the day. Panics if 24 or greater.
    #[inline]
    pub const fn hours(self, hours: u8) -> TimeBuilder<u8, ()> {
        match self.hours_checked(hours) {
            Some(next) => next,
            None => panic!("hours out of range (must be 0..24)"),
        }
    }

    /// Insert the hour of the day. Returns [`None`] if 24 or greater.
    #[inline]
    pub const fn hours_checked(self, hours: u8) -> Option<TimeBuilder<u8, ()>> {
        if 23 < hours {
            None
        } else {
            Some(TimeBuilder { hours, mins: () })
        }
    }
}

impl TimeBuilder<u8, ()> {
    /// Insert the minute of the hour. Panics if 60 or greater.
    #[inline]
    pub const fn minutes(self, mins: u8) -> TimeBuilder<u8, u8> {
        match self.minutes_checked(mins) {
            Some(next) => next,
            None => panic!("minutes out of range (must be 0..60)"),
        }
    }

    /// Insert the minute of the day. Returns [`None`] if 60 or greater.
    #[inline]
    pub const fn minutes_checked(self, mins: u8) -> Option<TimeBuilder<u8, u8>> {
        if 59 < mins {
            None
        } else {
            Some(TimeBuilder {
                hours: self.hours,
                mins,
            })
        }
    }
}

impl TimeBuilder<u8, u8> {
    /// Insert the number of seconds toward the next minute. Panics if 60 or greater.
    #[inline]
    pub const fn seconds(self, seconds: u8) -> Time {
        match self.seconds_checked(seconds) {
            Some(next) => next,
            None => panic!("seconds out of range (must be 0..60)"),
        }
    }

    ///  Insert the number of seconds toward the next minute. Returns [`None`] if 60 or greater.
    #[inline]
    pub const fn seconds_checked(self, seconds: u8) -> Option<Time> {
        if 59 < seconds {
            None
        } else {
            Some(Time::from_hms_saturating(self.hours, self.mins, seconds))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Time {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let hours = u.int_in_range(0..=23)?;
        let mins = u.int_in_range(0..=59)?;
        let seconds = u.int_in_range(0..=59)?;
        let nanos = u.int_in_range(0..=999_999_999)?;

        let t = time::Time::from_hms_nano(hours, mins, seconds, nanos)
            .expect("all values are in range");

        Ok(Self(t))
    }
}

impl fmt::Display for Time {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl Time {
    /// Assembles a [`Time`] from a number of hours, minutes and seconds. Internally normalizes
    /// the inputs, returning Self::MAX if hours ends up being outside the range '0..24'.
    pub const fn from_hms_saturating(mut hours: u8, mut mins: u8, mut seconds: u8) -> Self {
        // get seconds to below 1 minute, adding minutes as needed. adding to mins must be
        // saturating, otherwise an actual overflow could make an invalid Time appear valid.
        while seconds > 60 {
            seconds -= 60;
            mins = mins.saturating_add(1);
        }

        // similarly, get mins below 1 hour.
        while mins > 60 {
            mins -= 60;
            hours = hours.saturating_add(1);
        }

        match time::Time::from_hms(hours, mins, seconds) {
            Ok(t) => Self(t),
            // not sure if this branch is possible with the normalized inputs
            _ if hours < 12 => Self::MIN,
            _ => Self::MAX,
        }
    }

    const fn const_cmp(&self, other: &Self) -> Ordering {
        macro_rules! inner_cmp {
            ($fn_name:ident => $sel:expr, $other:expr) => {{
                if $sel.$fn_name() > $other.$fn_name() {
                    return Ordering::Greater;
                } else if $sel.$fn_name() < $other.$fn_name() {
                    return Ordering::Less;
                }
            }};
        }

        inner_cmp!(hour => self, other);
        inner_cmp!(minute => self, other);
        inner_cmp!(whole_second => self, other);
        inner_cmp!(subsec_nanos => self, other);

        // if none of those returned, we're equal
        debug_assert!(self.const_eq(other));
        Ordering::Equal
    }

    const fn const_eq(&self, other: &Self) -> bool {
        self.hour() == other.hour()
            && self.minute() == other.minute()
            && self.whole_second() == other.whole_second()
            && self.subsec_nanos() == other.subsec_nanos()
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.const_eq(other)
    }
}

impl Eq for Time {}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(other)
    }
}

impl Time {
    /// The minimum valid time, '00:00:00'.
    pub const MIN: Self = Self(time::Time::MIDNIGHT);

    /// The maximum valid time, '23:59:59.999999999'.
    pub const MAX: Self =
        Self(unsafe { time::Time::__from_hms_nanos_unchecked(23, 59, 59, 999_999_999) });
}

/*
/// A [`std::iter::Step`] helper for a [`Time`], allowing steps with a defined [`Duration`] interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeStepper(pub Time, pub crate::Duration);

impl<T> From<(T, crate::Duration)> for TimeStepper
where
    T: Into<Time>,
{
    fn from(tup: (T, crate::Duration)) -> Self {
        TimeStepper(tup.0.into(), tup.1)
    }
}

impl std::iter::Step for TimeStepper {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        // it doesn't make sense to give a number of steps if the step sizes are different.
        if start.1 != end.1 {
            return None;
        }

        let ratio = (end.0 - start.0).abs() / start.1.abs();

        Some(ratio.round() as usize)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0 + count as u32 * start.1, start.1))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0 - count as u32 * start.1, start.1))
    }
}
*/
#[test]
fn test_time_consts() {
    assert_eq!(Time::MIN - time::Duration::NANOSECOND, Time::MAX);
}

impl Time {
    /// The base length of a formatted [`Time`], without any fractional seconds.
    pub const BASE_STR_LENGTH: usize = "00:00:00".len();

    /// Adds an offset duration, saturating at the bounds of [`Time::MIN`]/[`Time::MAX`].
    pub fn saturating_add(self, offset: time::Duration) -> Self {
        // if the offset is negative, defer to 'saturating_sub' so we correctly saturate at
        // [`Time::MIN`] instead of [`Time::MAX`].
        if offset.is_negative() {
            return self.saturating_sub(offset.abs());
        }

        match self + offset {
            naive if naive < self => Self::MAX,
            naive => naive,
        }
    }

    /// Formats 'self' into an owned [`String`]. Infallible, unlike the [`fmt::Write`] methods.
    #[allow(dead_code)]
    pub fn append_to_string(&self, dst: &mut String, frac_digits: usize) {
        let mut buf = itoa::Buffer::new();
        let cap = if frac_digits == 0 {
            Self::BASE_STR_LENGTH
        } else {
            Self::BASE_STR_LENGTH + frac_digits + 1 // extra 1 for the "."
        };

        dst.reserve(cap);

        macro_rules! ensure_2_digits {
            ($field:expr) => {{
                let s = buf.format($field);
                if s.len() == 1 {
                    dst.push('0');
                }
                dst.push_str(s);
            }};
        }

        let (h, m, s) = self.0.as_hms();

        ensure_2_digits!(h);
        dst.push(':');
        ensure_2_digits!(m);
        dst.push(':');
        ensure_2_digits!(s);

        if frac_digits > 0 {
            let mut frac = self.0.nanosecond();

            for _ in 0..frac_digits.min(9) {
                frac /= 10;
            }

            dst.push('.');
            dst.push_str(buf.format(frac));
        }
    }

    pub(crate) const fn from_time(time: time::Time) -> Time {
        Time(time)
    }

    pub(crate) const fn into_time(self) -> time::Time {
        self.0
    }

    /// Subtracts an offset duration, saturating at the bounds of [`Time::MIN`]/[`Time::MAX`].
    pub fn saturating_sub(self, offset: time::Duration) -> Self {
        // if the offset is negative, defer to 'saturating_add' so we correctly saturate at
        // [`Time::MAX`] instead of [`Time::MIN`].
        if offset.is_negative() {
            return self.saturating_add(offset.abs());
        }

        match self - offset {
            naive if naive > self => Self::MIN,
            naive => naive,
        }
    }

    /// Returns a wrapped [`Time`], that overrides the default [`fmt::Display`] impl to
    /// omit seconds, only writing out 'HH:MM'.
    pub const fn fmt_hh_mm(&self) -> FmtHhMm<'_> {
        FmtHhMm(self)
    }

    /// Returns the hour component of this [`Time`]. In the range `0..24`.
    pub const fn hour(self) -> u8 {
        self.0.hour()
    }

    /// Returns the minute component of this [`Time`]. In the range `0..60`.
    pub const fn minute(self) -> u8 {
        self.0.minute()
    }

    /// Returns the number of whole seconds in this [`Time`]. In the range `0..60`.
    pub const fn whole_second(self) -> u8 {
        self.0.second()
    }

    /// Returns the fractional seconds (in nanoseconds) in this [`Time`].
    /// In the range `0..999_999_999`.
    pub const fn subsec_nanos(self) -> u32 {
        self.0.nanosecond()
    }

    /// Returns as a tuple of the number of hours, minutes and whole seconds in this [`Time`].
    pub const fn as_hms(self) -> (u8, u8, u8) {
        self.0.as_hms()
    }

    /// Returns as a tuple of the number of hours, minutes and floating poitn seconds in this
    /// [`Time`].
    pub const fn as_hms_frac(self) -> (u8, u8, f64) {
        let (hr, mn, sec, nano) = self.0.as_hms_nano();

        let frac_seconds = sec as f64 + (nano as f64 / super::conv::NANOS_PER_SECOND_F64);

        (hr, mn, frac_seconds)
    }
}

/// Formatting helper for [`Time`]. [`fmt::Display`] will write out the time in a 'HH:MM' format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FmtHhMm<'a>(&'a Time);

impl fmt::Display for FmtHhMm<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let (hours, mins, _) = self.0.as_hms();
        write!(formatter, "{hours:02}:{mins:02}")
    }
}

impl From<time::Time> for Time {
    fn from(time: time::Time) -> Self {
        Self(time)
    }
}

impl From<Time> for time::Time {
    fn from(time: Time) -> Self {
        time.0
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        time::Time::deserialize(deserializer).map(Time)
    }
}

impl ops::Add<time::Duration> for Time {
    type Output = Self;

    fn add(self, dur: time::Duration) -> Self {
        Self(self.0 + dur)
    }
}

impl ops::AddAssign<time::Duration> for Time {
    fn add_assign(&mut self, rhs: time::Duration) {
        self.0 += rhs;
    }
}

impl ops::Sub<time::Duration> for Time {
    type Output = Self;

    fn sub(self, dur: time::Duration) -> Self {
        Self(self.0 - dur)
    }
}

impl ops::SubAssign<time::Duration> for Time {
    fn sub_assign(&mut self, rhs: time::Duration) {
        self.0 -= rhs;
    }
}

impl ops::Sub for Time {
    type Output = time::Duration;

    fn sub(self, other: Self) -> Self::Output {
        self.0 - other.0
    }
}
