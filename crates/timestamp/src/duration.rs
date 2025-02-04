//! [`Duration`] definition and impls.
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::{fmt, ops};

use serde::{Deserialize, Serialize, de};

use crate::conv;
use crate::nanos::Nanos;
use crate::util::{clamp, max, min};

// the minimum/maximum values (inclusive) that the 'seconds' field in the inner
// [`protobuf::Duration`] allows, as both the target i64, and f64.
const MIN_SECONDS: i64 = -315_576_000_000;
const MAX_SECONDS: i64 = 315_576_000_000;

/// A duration, defined nearly identically to the well known protobuf type, [Duration], using a
/// signed whole number of seconds, and a signed sub-second number of nanoseconds.
///
/// Even though the nanosecond component is signed, it's wrapped in a new-type that enforces
/// it's never negative
///
/// [Duration]: <https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Duration>
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    /// The number of whole seconds in this duration. Must be between the minimum and maximum
    /// defined by the protobuf definition:
    /// '{ seconds | -315,576,000,000 <= seconds <= 315,576,000,000 }'
    seconds: i64,
    /// The subsecond number of nanoseconds in this duration. Must be between the minimum and
    /// maximum defined by the protobuf definition (which is enforced by the wrapper [`Nanos`])
    /// '{ nanos | 0 <= nanos <= 999,999,999 }'
    nanos: Nanos,
}

#[cfg(feature = "deepsize")]
deepsize::known_deep_size!(0; Duration);

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Duration {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let seconds = u.int_in_range(MIN_SECONDS..=MAX_SECONDS)?;
        let nanos = Nanos::arbitrary(u)?;
        Ok(Self { seconds, nanos })
    }
}

impl Default for Duration {
    /// Returns [`Duration::ZERO`].
    fn default() -> Self {
        Self::ZERO
    }
}

impl Duration {
    /// A zeroed duration.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::ZERO, 0 * Duration::SECOND);
    /// ```
    pub const ZERO: Self = Self {
        seconds: 0,
        nanos: Nanos::ZERO,
    };

    /// The minimum duration representable by the inner protobuf defined Duration.
    /// ```
    /// # use timestamp::Duration;
    /// // `Duration::from_seconds` saturates
    /// assert_eq!(Duration::MIN, Duration::from_seconds(i64::MIN));
    /// ```
    pub const MIN: Self = Self {
        seconds: MIN_SECONDS,
        nanos: Nanos::ZERO,
    };

    /// The maximum duration representable by the inner protobuf defined Duration.
    pub const MAX: Self = Self {
        seconds: MAX_SECONDS,
        nanos: Nanos::MAX,
    };

    /// A duration of 1 nanosecond.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::SECOND, 1_000_000_000 * Duration::NANOSECOND);
    /// ```
    pub const NANOSECOND: Self = Self {
        seconds: 0,
        nanos: unsafe { Nanos::new_unchecked(1) },
    };

    /// A duration of 1 microsecond.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::MICROSECOND, 1000 * Duration::NANOSECOND);
    /// ```
    pub const MICROSECOND: Self = Self {
        seconds: 0,
        nanos: unsafe { Nanos::new_unchecked(1_000) },
    };

    /// A duration of 1 millisecond.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::MILLISECOND, 1000 * Duration::MICROSECOND);
    /// ```
    pub const MILLISECOND: Self = Self {
        seconds: 0,
        nanos: unsafe { Nanos::new_unchecked(1_000_000) },
    };

    /// A duration of 1 second.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::SECOND, 1000 * Duration::MILLISECOND);
    /// ```
    pub const SECOND: Self = Self::from_seconds(1);

    /// A duration of 1 minute.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::MINUTE, 60 * Duration::SECOND);
    /// ```
    pub const MINUTE: Self = Self::from_seconds(60);

    /// A duration of 1 hour.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::HOUR, 60 * Duration::MINUTE);
    /// ```
    pub const HOUR: Self = Self::from_seconds(3600);

    /// A duration of 1 day.
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::DAY, 24 * Duration::HOUR);
    /// ```
    pub const DAY: Self = Self::from_seconds(86400);

    /// Returns the whole number of nanoseconds in this [`Duration`].
    pub const fn whole_nanoseconds(self) -> i128 {
        (self.seconds as i128 * conv::NANOS_PER_SECOND_I128) + self.nanos.get() as i128
    }

    /// Returns the whole number of milliseconds in this [`Duration`].
    pub const fn whole_milliseconds(self) -> i64 {
        (self.seconds * conv::MILLIS_PER_SECOND) + (self.nanos.get() as i64 / conv::NANOS_PER_MILLI)
    }

    /// Returns the whole number of microseconds in this [`Duration`].
    pub const fn whole_microseconds(self) -> i64 {
        (self.whole_nanoseconds() / 1000) as i64
    }

    /// Returns the number of sub-second nanoseconds.
    pub const fn subsec_nanoseconds(self) -> i32 {
        self.nanos.get()
    }

    pub(crate) const fn from_time(dur: time::Duration) -> Self {
        let (nanos, second_offset) = Nanos::new_overflow(dur.subsec_nanoseconds());

        Self {
            seconds: dur.whole_seconds() + second_offset,
            nanos,
        }
    }

    /// Returns the absolute duration. If the duration was already positive, this is a no-op.
    /// ```
    /// # use timestamp::Duration;
    /// let neg_second = -Duration::SECOND;
    /// assert_eq!(neg_second.abs(), Duration::SECOND);
    ///
    /// assert_eq!(Duration::SECOND.abs(), Duration::SECOND);
    /// ```
    pub const fn abs(self) -> Self {
        if self.is_negative() {
            self.const_neg()
        } else {
            self
        }
    }

    /// [`std::ops::Neg::neg`], but 'const'.
    /// ```
    /// # use timestamp::Duration;
    /// let dur = Duration::from_millis(12345);
    /// // negating twice should be identical to the input.
    /// assert_eq!(dur.const_neg().const_neg(), dur);
    /// ```
    pub const fn const_neg(self) -> Self {
        let neg_secs = -self.seconds;

        let (nanos, overflow) = self.nanos.overflowing_neg();

        Self {
            seconds: neg_secs + overflow,
            nanos,
        }
    }

    pub(crate) const fn into_time_duration(self) -> time::Duration {
        time::Duration::new(self.seconds, self.nanos.get())
    }

    pub(crate) const fn from_time_duration(dur: time::Duration) -> Self {
        Self::new(dur.whole_seconds(), dur.subsec_nanoseconds())
    }

    /// Returns the number of milliseconds in this duration.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(Duration::MILLISECOND.millis(), 1);
    ///
    /// assert_eq!(Duration::SECOND.millis(), 1000);
    /// ```
    pub const fn millis(self) -> i64 {
        (self.seconds * conv::MILLIS_PER_SECOND) + (self.nanos.get() as i64 / conv::NANOS_PER_MILLI)
    }

    /// Builds a duration from a whole number of milliseconds. Returns [`None`] if the number
    /// of seconds is out of range. If checked behavior isn't desired, the counterpart function
    /// [`Duration::from_millis`] saturates at [`Duration::MIN`] and [`Duration::MAX`].
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let one_second = Duration::from_millis_checked(1000).unwrap();
    /// assert_eq!(one_second, Duration::SECOND);
    /// ```
    pub const fn from_millis_checked(millis: i32) -> Option<Self> {
        Self::MILLISECOND.checked_mul(millis)
    }

    /// Builds a duration from a whole number of [`i64`] milliseconds. Returns [`None`] if the
    /// number of seconds is out of range.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let one_second = Duration::from_millis_i64_checked(1000).unwrap();
    /// assert_eq!(one_second, Duration::SECOND);
    ///
    /// let max_millis = Duration::MAX.millis();
    /// assert_eq!(Duration::from_millis_i64_checked(max_millis + 1), None);
    /// ```
    pub const fn from_millis_i64_checked(millis: i64) -> Option<Self> {
        let seconds = millis / conv::MILLIS_PER_SECOND;
        let subsec_millis = (millis % conv::MILLIS_PER_SECOND) as i32;

        Self::new_checked(seconds, subsec_millis * 1000)
    }

    /// Calls [`Duration::from_millis_i64_checked`], but returns [`Duration::MIN`] or
    /// [`Duration::MAX`] if 'millis' is out of range ('MIN' if 'millis' < 0, and 'MAX' otherwise)
    pub const fn from_millis_i64_saturating(millis: i64) -> Self {
        match Self::from_millis_i64_checked(millis) {
            Some(d) => d,
            None if millis.is_negative() => Self::MIN,
            None => Self::MAX,
        }
    }

    /// Builds a duration from a whole number of milliseconds. Saturates at [`Duration::MIN`] or
    /// [`Duration::MAX`] if the result is out of range. See the counterpart function
    /// [`Duration::from_millis_checked`] for checked behavior.
    /// ```
    /// # use timestamp::Duration;
    /// let one_second = Duration::from_millis(1000);
    /// assert_eq!(one_second, Duration::SECOND);
    /// ```
    pub const fn from_millis(millis: i32) -> Self {
        Self::MILLISECOND.saturating_mul(millis)
    }

    /// Builds a duration from a whole number of microseconds. Returns [`None`] if the number
    /// of seconds is out of range. If checked behavior isn't desired, the counterpart function
    /// [`Duration::from_micros`] saturates at [`Duration::MIN`] and [`Duration::MAX`].
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let one_second = Duration::from_micros_checked(1_000_000).unwrap();
    /// assert_eq!(one_second, Duration::SECOND);
    ///
    /// let way_over_max = Duration::from_micros_checked(i64::MAX);
    /// assert_eq!(way_over_max, None);
    /// ```
    pub const fn from_micros_checked(micros: i64) -> Option<Self> {
        let seconds = micros / conv::MICROS_PER_SECOND;
        let subsec_micros = (micros % conv::MICROS_PER_SECOND) as i32;

        Self::new_checked(seconds, subsec_micros * 1_000)
    }

    /// Builds a duration from a whole number of microseconds. Saturates at [`Duration::MIN`] or
    /// [`Duration::MAX`] if the result is out of range. See the counterpart function
    /// [`Duration::from_micros_checked`] for checked behavior.
    /// ```
    /// # use timestamp::Duration;
    /// let one_second = Duration::from_micros(1_000_000);
    /// assert_eq!(one_second, Duration::SECOND);
    ///
    /// let way_over_max = Duration::from_micros(i64::MAX);
    /// assert_eq!(way_over_max, Duration::MAX);
    /// ```
    pub const fn from_micros(micros: i64) -> Self {
        let seconds = micros / conv::MICROS_PER_SECOND;
        if seconds < MIN_SECONDS {
            Self::MIN
        } else if seconds > MAX_SECONDS {
            Self::MAX
        } else {
            let nanos =
                unsafe { Nanos::new_unchecked(((micros % conv::MICROS_PER_SECOND) * 1000) as i32) };

            Self { seconds, nanos }
        }
    }

    // const unsafe functions throw this warning. For non-const unsafe functions, explicit unsafe
    // blocks are the desired behavior, as it should be here.
    #[allow(unused_unsafe)]
    /// Creates a new [`Duration`] with no bound checks or conversion on the inner [`Nanos`].
    ///
    /// # Safety
    /// 'seconds' must be in the range '-315_576_000_000..=315_576_000_000'.
    /// 'nanos' must be in the range '0..=999_999_999'.
    ///
    /// If either of these are violated, there will not be any undefined behavior, but math will
    /// likely be incorrect, and [`PartialOrd`]/[`Ord`]/[`PartialEq`]/[`Eq`]/[`Hash`]
    /// implementations will not behave as expected.
    pub const unsafe fn new_unchecked(seconds: i64, nanos: i32) -> Self {
        Self {
            seconds,
            #[allow(unsafe_op_in_unsafe_fn)]
            nanos: unsafe { Nanos::new_unchecked(nanos) },
        }
    }

    /// Returns `true` if this duration is a negative amount of time.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// assert!(!Duration::SECOND.is_negative());
    /// assert!((-Duration::SECOND).is_negative());
    /// ```
    pub const fn is_negative(&self) -> bool {
        self.seconds.is_negative()
    }

    /// Returns `true` if this duration is a positive amount of time.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// assert!(Duration::SECOND.is_positive());
    /// assert!(!(-Duration::SECOND).is_positive());
    /// ```
    pub const fn is_positive(&self) -> bool {
        self.seconds.is_positive() || self.nanos.get().is_positive()
    }

    /// Rounds to the nearest full second.
    /// ```
    /// # use timestamp::Duration;
    /// let nearly_1_sec = Duration::from_seconds_f64(0.9);
    /// assert_eq!(nearly_1_sec.round(), Duration::SECOND);
    ///
    /// let half_second = Duration::from_millis(500);
    /// assert_eq!(half_second.round(), Duration::SECOND);
    ///
    /// let just_under_half = Duration::from_millis(499);
    /// assert_eq!(just_under_half.round(), Duration::ZERO);
    /// ```
    pub const fn round(self) -> Self {
        Self {
            seconds: self.seconds + self.nanos.round(),
            nanos: Nanos::ZERO,
        }
    }

    /// Returns `true` if this duration is zero.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// assert!(!Duration::SECOND.is_zero());
    /// assert!(Duration::ZERO.is_zero());
    /// ```
    pub const fn is_zero(&self) -> bool {
        Self::ZERO.const_cmp(self).is_eq()
    }

    /// Internal helper for disassembling a Duration
    #[inline]
    pub(crate) const fn into_parts(self) -> (i64, Nanos) {
        (self.seconds, self.nanos)
    }

    /// Internal helper for assembling a Duration
    #[inline]
    pub(crate) const fn from_parts(seconds: i64, nanos: Nanos) -> Self {
        Self { seconds, nanos }
    }

    /// Constructs a [`Duration`] from a number of seconds and nanoseconds.
    /// If out of bounds, this function clamps the number of seconds to a valid value.
    ///
    /// See [`Duration::new_checked`] if clamping is not the desired behavior.
    pub const fn new(seconds: i64, nanos: i32) -> Self {
        let (nanos, overflow_secs) = Nanos::new_overflow(nanos);
        Self {
            nanos,
            seconds: clamp!(seconds + overflow_secs; MIN_SECONDS..=MAX_SECONDS),
        }
    }

    /// Creates a new [`Duration`], returning [`None`] if the duration is out of range.
    pub const fn new_checked(mut seconds: i64, nanos: i32) -> Option<Self> {
        let (nanos, overflow_secs) = Nanos::new_overflow(nanos);

        seconds += overflow_secs;

        // if seconds is in the valid range, we can build and return the timestamp.
        if MIN_SECONDS <= seconds && seconds <= MAX_SECONDS {
            Some(Self { seconds, nanos })
        } else {
            None
        }
    }

    /// Assembles a [`Duration`] from a given number of seconds. If the value is out of range,
    /// it'll be clamped to the nearest bound. If this behavior is not wanted, use
    /// [`Duration::new_checked`].
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let two_secs = Duration::from_seconds(2);
    /// assert_eq!(two_secs, 2 * Duration::SECOND);
    ///
    /// let out_of_range = Duration::MAX.whole_seconds() + 10;
    /// let clamped = Duration::from_seconds(out_of_range);
    ///
    /// // Duration::MAX has a nanosecond component as well, so we need to ignore those
    /// // with 'Duration::whole_seconds'.
    /// assert_eq!(clamped.whole_seconds(), Duration::MAX.whole_seconds());
    /// ```
    pub const fn from_seconds(secs: i64) -> Self {
        Self {
            seconds: clamp!(secs; MIN_SECONDS..=MAX_SECONDS),
            nanos: Nanos::ZERO,
        }
    }

    /// Assembles a [`Duration`] from a given number of floating point seconds. If the value is
    /// out of range, it'll be clamped to the nearest bound. If this behavior is not wanted, use
    /// [`Duration::from_seconds_f64_checked`].
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let dur = Duration::from_seconds_f64(1.5);
    /// assert_eq!(dur, 1.5 * Duration::SECOND);
    ///
    /// let overflow = Duration::MAX.as_seconds_f64() + 10.0;
    /// assert_eq!(Duration::from_seconds_f64(overflow), Duration::MAX);
    /// ```
    pub fn from_seconds_f64(secs: f64) -> Self {
        let mut seconds = secs.trunc() as i64;
        let nanos_f64 = secs.fract() * conv::NANOS_PER_SECOND_F64;

        let (nanos, overflow) = Nanos::new_overflow(nanos_f64 as i32);
        seconds += overflow;

        let min = Self::MIN;
        let max = Self::MAX;

        clamp!(Self { seconds, nanos }; min..=max)
    }

    /// Assembles a [`Duration`] from a given number of floating point seconds. If the value is
    /// out of range, this will return [`None`].
    pub fn from_seconds_f64_checked(secs: f64) -> Option<Self> {
        let mut seconds = secs.trunc() as i64;
        let nanos_f64 = secs.fract() * conv::NANOS_PER_SECOND_F64;

        let (nanos, overflow) = Nanos::new_overflow(nanos_f64 as i32);

        seconds += overflow;

        if (MIN_SECONDS..MAX_SECONDS).contains(&seconds) {
            Some(Self { seconds, nanos })
        } else {
            None
        }
    }

    /// Assembles a [`Duration`] from a whole number of minutes. Saturates if the number of
    /// minutes surpasses the maximum supported duration.
    pub const fn from_minutes(mins: i64) -> Self {
        Self {
            seconds: clamp!(mins.saturating_mul(60); MIN_SECONDS..=MAX_SECONDS),
            nanos: Nanos::ZERO,
        }
    }

    /// Assembles a [`Duration`] from a whole number of nanoseconds. Saturates if the number of
    /// minutes surpasses the maximum supported duration.
    pub const fn from_nanos(nanos: i128) -> Self {
        let seconds = (nanos / conv::NANOS_PER_SECOND_I128) as i64;
        let (nanos, overflow) = Nanos::new_overflow((nanos % conv::NANOS_PER_SECOND_I128) as i32);

        // verifying that the math checks out
        #[cfg(debug_assertions)]
        if overflow != 0 {
            panic!("Duration::from_nanos had overflow when computing subsecond nanos");
        }

        Self {
            seconds: seconds + overflow,
            nanos,
        }
    }

    /// Returns the number of whole seconds in this duration.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let dur_1 = Duration::new(0, 999_999_999);
    /// assert_eq!(dur_1.whole_seconds(), 0);
    ///
    /// let dur_2 = 1.5_f64 * Duration::SECOND;
    /// assert_eq!(dur_2.whole_seconds(), 1);
    ///
    /// let dur_3 = 1.9999999999999_f64 * Duration::SECOND;
    /// assert_eq!(dur_3.whole_seconds(), 1);
    /// ```
    #[inline]
    pub const fn whole_seconds(self) -> i64 {
        self.seconds
    }

    /// Returns the number of whole hours in this duration.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let dur_1 = 30_i32 * Duration::MINUTE;
    /// assert_eq!(dur_1.whole_hours(), 0);
    ///
    /// let dur_2 = 123_i32 * Duration::MINUTE;
    /// assert_eq!(dur_2.whole_hours(), 2);
    /// ```
    #[inline]
    pub const fn whole_hours(self) -> i64 {
        self.seconds / 3600
    }

    /// Returns the number of whole minutes in this duration.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let dur_1 = 30_i32 * Duration::MINUTE;
    /// assert_eq!(dur_1.whole_minutes(), 30);
    ///
    /// let dur_2 = 1.5_f64 * Duration::MINUTE;
    /// assert_eq!(dur_2.whole_minutes(), 1);
    /// ```
    #[inline]
    pub const fn whole_minutes(self) -> i64 {
        self.seconds / 60
    }

    /// Returns the duration in floating point seconds.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let dur_1 = 30_i32 * Duration::SECOND;
    /// assert!((dur_1.as_seconds_f64() - 30.0).abs() < 1e-8_f64);
    ///
    /// let dur_2 = 1.5_f64 * Duration::MINUTE;
    /// assert!((dur_2.as_seconds_f64() - 90.0).abs() < 1e-8_f64);
    /// ```
    #[inline]
    pub const fn as_seconds_f64(self) -> f64 {
        self.seconds as f64 + (self.nanos.get() as f64 / conv::NANOS_PER_SECOND_F64)
    }

    /// Formatting helper. Wraps a reference to the duration in a type that implements
    /// [`fmt::Display`], writing the duration in 'HH:MM' format.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let fmt = (30_i32 * Duration::MINUTE).fmt_hh_mm().to_string();
    /// assert_eq!(fmt.as_str(), "00:30");
    /// ```
    #[inline]
    pub const fn fmt_hh_mm(&self) -> DurHrMin<'_> {
        DurHrMin(self)
    }

    /// Formatting helper. Wraps a reference to the duration in a type that implements
    /// [`fmt::Display`], writing the duration in 'HH:MM:SS' format.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let fmt = (30_i32 * Duration::MINUTE + Duration::SECOND)
    ///     .fmt_hh_mm_ss()
    ///     .to_string();
    /// assert_eq!(fmt.as_str(), "00:30:01");
    /// ```
    #[inline]
    pub const fn fmt_hh_mm_ss(&self) -> DurHrMinSec<'_> {
        DurHrMinSec(self)
    }

    /// Formatting helper. Wraps a reference to the duration in a type that implements
    /// [`fmt::Display`], writing the duration in 'HH:MM:SS.FFFF' format.
    #[inline]
    pub const fn fmt_hh_mm_ss_ff(&self) -> DurHrMinSecFrac<'_> {
        DurHrMinSecFrac(self)
    }

    /// Adds 2 [`Duration`]s, checking to make sure the resulting [`Duration`] is in the valid
    /// range of values.
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        Self::new_checked(
            self.seconds + rhs.seconds,
            self.nanos.get() + rhs.nanos.get(),
        )
    }

    /// Multiplies a [`Duration`] a whole number of times.
    /// ```
    /// # use timestamp::Duration;
    /// let five_seconds = Duration::SECOND
    ///     .checked_mul(5)
    ///     .expect("5 seconds is well below the max");
    ///
    /// assert_eq!(five_seconds.whole_seconds(), 5);
    /// assert_eq!(five_seconds.subsec_nanoseconds(), 0);
    ///
    /// let half_max = Duration::MAX / 2_i32;
    /// let overflow = half_max.checked_mul(3);
    /// assert_eq!(overflow, None);
    /// ```
    #[inline]
    pub const fn checked_mul(self, rhs: i32) -> Option<Self> {
        let (new_nanos, overflow) = self.nanos.overflowing_mul(rhs);

        let new_secs = self
            .seconds
            .saturating_mul(rhs as i64)
            .saturating_add(overflow);

        if MIN_SECONDS <= new_secs && new_secs <= MAX_SECONDS {
            Some(Self {
                seconds: new_secs,
                nanos: new_nanos,
            })
        } else {
            None
        }
    }

    /// Multiplies a [`Duration`] a whole number of times, saturating at the minimum or maximum
    /// in the case of an overflow.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// let nearly_max = Duration::MAX - Duration::SECOND;
    /// let saturated = nearly_max.saturating_mul(2);
    /// assert_eq!(saturated, Duration::MAX);
    /// ```
    #[inline]
    pub const fn saturating_mul(self, rhs: i32) -> Self {
        let (nanos, overflow) = self.nanos.overflowing_mul(rhs);

        let seconds = self
            .seconds
            .saturating_mul(rhs as i64)
            .saturating_add(overflow);

        if seconds > MAX_SECONDS {
            Self::MAX
        } else if seconds < MIN_SECONDS {
            Self::MIN
        } else {
            Self { seconds, nanos }
        }
    }

    /// Adds 2 [`Duration`]s, saturating at the bound if the result is out of range.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self::new(
            self.seconds + rhs.seconds,
            self.nanos.get() + rhs.nanos.get(),
        )
    }

    /// Subtracts 2 [`Duration`]s, checking to make sure the resulting [`Duration`] is in the
    /// valid range of values.
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        Self::new_checked(
            self.seconds - rhs.seconds,
            self.nanos.get() - rhs.nanos.get(),
        )
    }

    /// Subtracts 2 [`Duration`]s, saturating at the bound if the result is out of range.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// assert_eq!(
    ///     Duration::MIN,
    ///     Duration::MIN.saturating_sub(Duration::MINUTE)
    /// );
    /// ```
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self::new(
            self.seconds - rhs.seconds,
            self.nanos.get() - rhs.nanos.get(),
        )
    }

    /// A const [`Ord::cmp`]. Called by both the [`PartialOrd`] and [`Ord`] implementations.
    ///
    /// ```
    /// # use timestamp::Duration;
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(Duration::ZERO.const_cmp(&Duration::ZERO), Ordering::Equal);
    ///
    /// assert_eq!(Duration::MIN.const_cmp(&Duration::MAX), Ordering::Less);
    ///
    /// assert_eq!(
    ///     Duration::MINUTE.const_cmp(&Duration::ZERO),
    ///     Ordering::Greater
    /// );
    /// ```
    #[inline]
    pub const fn const_cmp(&self, other: &Self) -> Ordering {
        if self.seconds > other.seconds {
            Ordering::Greater
        } else if self.seconds < other.seconds {
            Ordering::Less
        } else {
            self.nanos.const_cmp(other.nanos)
        }
    }
}

/// Duration formatting wrapper that writes as a 'HH:MM' format via [`fmt::Display`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DurHrMin<'a>(&'a Duration);

impl fmt::Display for DurHrMin<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut mins = self.0.whole_minutes() % 60;

        // handle rounding up, since we don't show seconds.
        if self.0.seconds % 60 >= 30 {
            mins += 1;
        }

        write!(formatter, "{:02}:{:02}", self.0.whole_hours(), mins)
    }
}

/// Duration formatting wrapper that writes as a 'HH:MM:SS' format via [`fmt::Display`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DurHrMinSec<'a>(&'a Duration);

impl fmt::Display for DurHrMinSec<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:02}:{:02}:{:02}",
            self.0.whole_hours(),
            self.0.whole_minutes() % 60,
            self.0.whole_seconds() % 60,
        )
    }
}

/// Duration formatting wrapper that writes as a 'HH:MM:SS.FF' format via [`fmt::Display`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DurHrMinSecFrac<'a>(&'a Duration);

impl fmt::Display for DurHrMinSecFrac<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:02}:{:02}:{:02}",
            self.0.whole_hours(),
            self.0.whole_minutes() % 60,
            (self.0.as_seconds_f64() % 60.0),
        )
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(other)
    }

    fn max(self, other: Self) -> Self {
        max!(self, other)
    }

    fn min(self, other: Self) -> Self {
        min!(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        clamp!(self; min..=max)
    }
}

impl Eq for Duration {}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.seconds == other.seconds && self.nanos == other.nanos
    }
}

impl Hash for Duration {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_i64(self.seconds);
        state.write_i32(self.nanos.get());
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:02}:{:02}:{:02}",
            self.whole_hours(),
            self.whole_minutes() % 60,
            self.as_seconds_f64() % 60.0,
        )
    }
}

impl From<time::Duration> for Duration {
    fn from(d: time::Duration) -> Self {
        Self::from_time(d)
    }
}

impl From<std::time::Duration> for Duration {
    fn from(std_dur: std::time::Duration) -> Self {
        let seconds = std_dur.as_secs() as i64;
        let (nanos, overflow) = Nanos::new_overflow(std_dur.subsec_nanos() as i32);

        Self {
            seconds: seconds + overflow,
            nanos,
        }
    }
}

impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> Self {
        std::time::Duration::new(d.seconds as u64, d.nanos.get() as u32)
    }
}

impl From<Duration> for time::Duration {
    fn from(d: Duration) -> Self {
        d.into_time_duration()
    }
}

impl ops::Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let (new_nanos, overflow) = self.nanos.overflowing_add(other.nanos);
        let new_secs = self.seconds + other.seconds + overflow;

        Self {
            seconds: new_secs,
            nanos: new_nanos,
        }
    }
}

impl ops::AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl ops::Sub for Duration {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        let (new_nanos, overflow) = self.nanos.overflowing_sub(other.nanos);
        let new_secs = (self.seconds - other.seconds) + overflow;

        Self {
            seconds: new_secs,
            nanos: new_nanos,
        }
    }
}

impl ops::SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

macro_rules! impl_int_mul_div {
    ($($t:ty),* $(,)?) => {
        $(
            impl ops::Mul<$t> for Duration {
                type Output = Self;

                fn mul(self, rhs: $t) -> Self::Output {
                    let seconds = self.seconds * rhs as i64;
                    let (nanos, overflow) = self.nanos.overflowing_mul(rhs as i32);

                    Self { seconds: seconds + overflow, nanos }
                }
            }

            impl ops::Mul<Duration> for $t {
                type Output = Duration;

                fn mul(self, rhs: Duration) -> Self::Output {
                    rhs * self
                }
            }

            impl ops::MulAssign<$t> for Duration {
                fn mul_assign(&mut self, rhs: $t) {
                    *self = *self * rhs;
                }
            }

            impl ops::Div<$t> for Duration {
                type Output = Self;

                fn div(self, rhs: $t) -> Self::Output {
                    let seconds = self.seconds * rhs as i64;
                    let (nanos, overflow) = self.nanos.overflowing_div(rhs as i32);

                    Self { seconds: seconds + overflow, nanos }
                }
            }

            impl ops::DivAssign<$t> for Duration {
                fn div_assign(&mut self, rhs: $t) {
                    *self = *self / rhs;
                }
            }
        )*
    };
}

macro_rules! impl_float_mul_div {
    ($($t:ty),* $(,)?) => {
        $(
            impl ops::Mul<$t> for Duration {
                type Output = Self;

                fn mul(self, rhs: $t) -> Self::Output {
                    Self::from_seconds_f64(self.as_seconds_f64() * rhs as f64)
                }
            }

            impl ops::Mul<Duration> for $t {
                type Output = Duration;

                fn mul(self, rhs: Duration) -> Self::Output {
                    rhs * self
                }
            }

            impl ops::Div<$t> for Duration {
                type Output = Self;

                fn div(self, rhs: $t) -> Self::Output {
                    let seconds = (self.seconds as f64 / rhs as f64) as i64;
                    let (nanos, overflow) = self.nanos.overflowing_div_f64(rhs as f64);

                    Self { seconds: seconds + overflow, nanos }
                }
            }
        )*
    };
}

impl_int_mul_div!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_float_mul_div!(f32, f64);

impl ops::Div for Duration {
    type Output = f64;

    fn div(self, rhs: Self) -> f64 {
        self.as_seconds_f64() / rhs.as_seconds_f64()
    }
}

impl ops::Neg for Duration {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        let (nanos, overflow) = self.nanos.overflowing_neg();
        Self {
            seconds: overflow - self.seconds,
            nanos,
        }
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DurationVisitor)
    }
}

struct DurationVisitor;

impl de::Visitor<'_> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a duration, as a formatted string or floating point seconds"
        )
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Duration::from_seconds_f64_checked(v) {
            Some(dur) => Ok(dur),
            None => Err(de::Error::invalid_value(
                de::Unexpected::Float(v),
                &"out of range",
            )),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Duration::from_seconds(v as i64))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Duration::from_seconds(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<Duration>().map_err(de::Error::custom)
    }
}

/// The [`FromStr::Err`] for [`Duration`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidDuration {
    /// An error encountered when trying to parse an empty string
    #[error("empty string not a valid duration")]
    EmptyString,
    /// An error encountered when trying to parse invalid characters
    #[error("found invalid char in duration: '{0}'")]
    InvalidChar(char),
    /// An error encountered when the duration has more than 3 components (for HH:MM:SS)
    #[error("too many ':' separated components: {0} found")]
    TooManyComponents(usize),
    /// An error encountered when a floating point value is invalid
    #[error(transparent)]
    InvalidFloat(#[from] std::num::ParseFloatError),
    /// An error encountered when an integer value is invalid
    #[error(transparent)]
    InvalidInt(#[from] std::num::ParseIntError),
}

impl FromStr for Duration {
    type Err = InvalidDuration;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(InvalidDuration::EmptyString);
        }

        let mut has_frac = false;

        // check for invalid characters (and also '.' to denote we have fractional seconds)
        for ch in trimmed.chars() {
            match ch {
                '.' => has_frac = true,
                ':' | '0'..='9' => (),
                bad_char => return Err(InvalidDuration::InvalidChar(bad_char)),
            }
        }

        // multipliers to convert a component at an index (0 being seconds, 1 being minutes, etc)
        // to seconds.
        const UNIT_MULTS: [i64; 4] = [1, 60, 3600, 86400];

        // macro handles parsing the components to a number type, that way we can
        // parse integers only if there was no '.' found (meaning no fractional seconds).
        macro_rules! parse_string {
            ($s:expr => $fn_name:ident($second_ty:ty)) => {{
                let mut seconds = <$second_ty>::default();

                for (idx, component) in $s.rsplit(':').enumerate() {
                    let num = component.parse::<$second_ty>()?;

                    seconds += match UNIT_MULTS.get(idx) {
                        Some(mult) => *mult as $second_ty * num,
                        None => return Err(InvalidDuration::TooManyComponents(idx)),
                    };
                }

                return Ok(Duration::$fn_name(seconds));
            }};
        }

        if has_frac {
            parse_string!(trimmed => from_seconds_f64(f64));
        } else {
            parse_string!(trimmed => from_seconds(i64));
        }
    }
}

pub mod serde_as_seconds {
    //! Module for use with [`serde`]'s field attribute, `#[serde(with = "serde_as_seconds")]`.
    //! Using this module serializes a [`Duration`] as the number of floating point seconds, and
    //! expects the same when deserializing.

    use serde::{Deserialize, Deserializer, Serializer};

    use super::Duration;

    #[allow(missing_docs)]
    pub fn serialize<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(dur.as_seconds_f64())
    }

    #[allow(missing_docs)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = f64::deserialize(deserializer)?;
        match Duration::from_seconds_f64_checked(s) {
            Some(dur) => Ok(dur),
            None => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Float(s),
                &"number of seconds out of range for a 'Duration'",
            )),
        }
    }
}

pub mod serde_as_seconds_option {
    //! Module for use with [`serde`]'s field attribute, `#[serde(with = "serde_as_seconds")]`.
    //! Identical to [`serde_as_seconds`], but serializes/deserializes to and from
    //! [`Option<Duration>`] instead.
    //!
    //! [`serde_as_seconds`]: [`super::serde_as_seconds`]

    use serde::{Deserialize, Deserializer, Serializer};

    use super::Duration;

    #[allow(missing_docs)]
    pub fn serialize<S>(dur: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *dur {
            Some(dur) => serializer.serialize_some(&dur.as_seconds_f64()),
            None => serializer.serialize_none(),
        }
    }

    #[allow(missing_docs)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<f64>::deserialize(deserializer)? {
            Some(seconds) => match Duration::from_seconds_f64_checked(seconds) {
                Some(dur) => Ok(Some(dur)),
                None => Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Float(seconds),
                    &"number of seconds out of range for a 'Duration'",
                )),
            },
            None => Ok(None),
        }
    }
}

#[cfg(feature = "prost")]
impl From<prost_types::Duration> for Duration {
    fn from(dur: prost_types::Duration) -> Self {
        let (nanos, overflow) = Nanos::new_overflow(dur.nanos);
        Self {
            seconds: dur.seconds + overflow,
            nanos,
        }
    }
}

#[cfg(feature = "prost")]
impl From<Duration> for prost_types::Duration {
    fn from(dur: Duration) -> Self {
        Self {
            seconds: dur.seconds,
            nanos: dur.nanos.get(),
        }
    }
}

#[cfg(feature = "rand")]
mod rand_impls {
    use super::Duration;
    use crate::nanos::Nanos;

    #[cfg(feature = "rand")]
    impl rand::distributions::Distribution<Duration> for rand::distributions::Standard {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Duration {
            rng.gen_range(Duration::MIN..=Duration::MAX)
        }
    }

    impl rand::distributions::uniform::SampleUniform for Duration {
        type Sampler = DurationSampler;
    }

    pub struct DurationSampler {
        min: Duration,
        delta: Duration,
    }

    impl DurationSampler {
        fn new_from(min: Duration, max: Duration) -> Self {
            Self {
                delta: max - min,
                min,
            }
        }
    }

    impl rand::distributions::uniform::UniformSampler for DurationSampler {
        type X = Duration;

        fn new<B1, B2>(low: B1, high: B2) -> Self
        where
            B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
            B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        {
            Self::new_from(*low.borrow(), *high.borrow())
        }

        fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
        where
            B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
            B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        {
            Self::new_from(
                *low.borrow(),
                high.borrow().saturating_add(Duration::NANOSECOND),
            )
        }

        fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
            let seconds = rng.gen_range(0..=self.delta.seconds);
            let nanos = rng.gen_range(Nanos::ZERO..=self.delta.nanos);
            self.min + Duration { seconds, nanos }
        }
    }
}
