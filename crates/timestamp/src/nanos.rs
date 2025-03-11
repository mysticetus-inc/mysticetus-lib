//! A Newtype wrapper around [`i32`] that represents subsecond nanoseconds.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::conv;

const MAX_NANOS: i32 = 999_999_999;

const HALF_SEC_NANOS: i32 = 500_000_000;

/// A newtype around [`i32`] representing a number of subsecond nanoseconds. Used to enforce
/// the invariant in [`crate::Duration`] that the nanos must be
#[allow(clippy::derived_hash_with_manual_eq)]
// `PartialEq` has expected behavior, and is only manually implemented for 'const'.
#[derive(Debug, Clone, Copy, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
pub struct Nanos(i32);

#[cfg(feature = "deepsize")]
deepsize::known_deep_size!(0; Nanos);

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Nanos {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.int_in_range(0..=MAX_NANOS).map(Self)
    }
}

impl Default for Nanos {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Nanos {
    /// 0 Nanoseconds.
    /// ```
    /// # use timestamp::nanos::Nanos;
    /// assert_eq!(Nanos::ZERO.get(), 0);
    /// ```
    pub const ZERO: Self = Self(0);

    /// The maximum number of nanoseconds allowed by this type.
    /// ```
    /// # use timestamp::nanos::Nanos;
    /// let one_nano = Nanos::new(1).unwrap();
    /// let (nanos, overflow_secs) = Nanos::MAX.overflowing_add(one_nano);
    ///
    /// assert_eq!(nanos.get(), 0);
    /// assert_eq!(overflow_secs, 1);
    /// ```
    pub const MAX: Self = Self(MAX_NANOS);

    /// Creates a new [`Nanos`] wrapper, returning [`None`] if 'nanos' is out of the range
    /// '0..=999_999_999'.
    pub const fn new(nanos: i32) -> Option<Self> {
        if 0 <= nanos && nanos <= MAX_NANOS {
            Some(Self(nanos))
        } else {
            None
        }
    }

    /// ```
    /// # use timestamp::nanos::Nanos;
    /// let (negative, neg_overflow) = Nanos::new_overflow(-1000);
    /// let wrapped = Nanos::new(1_000_000_000 - 1000).unwrap();
    /// assert_eq!(negative, wrapped);
    /// assert_eq!(neg_overflow, -1);
    ///
    /// let (valid_nanos, valid_overflow) = Nanos::new_overflow(1000);
    /// assert_eq!(valid_nanos, Nanos::new(1000).unwrap());
    /// assert_eq!(valid_overflow, 0);
    ///
    /// let (max_nanos, max_overflow) = Nanos::new_overflow(999_999_999);
    /// assert_eq!(max_nanos, Nanos::MAX);
    /// assert_eq!(max_overflow, 0);
    ///
    /// let (one_sec, exact_overflow) = Nanos::new_overflow(1_000_000_000);
    /// assert_eq!(one_sec, Nanos::ZERO);
    /// assert_eq!(exact_overflow, 1);
    ///
    /// let (over_one_sec, overflow) = Nanos::new_overflow(1_000_000_001);
    /// let expected = Nanos::new(1).unwrap();
    /// assert_eq!(over_one_sec, expected);
    /// assert_eq!(overflow, 1);
    /// ```
    #[inline]
    pub const fn new_overflow(nanos: i32) -> (Self, i64) {
        let mut seconds = nanos / conv::NANOS_PER_SECOND_I32;
        let mut rem_nanos = nanos - (seconds * conv::NANOS_PER_SECOND_I32);

        if rem_nanos.is_negative() {
            // take a second from here,
            seconds -= 1;
            // and add it to nanos so we end up in the right '0..=999_999_999' range.
            rem_nanos += conv::NANOS_PER_SECOND_I32;
        }

        // const-usable debug check to make sure that nanos is within range.
        if rem_nanos > MAX_NANOS || rem_nanos < 0 {
            #[allow(arithmetic_overflow)]
            let _ = i64::MAX + 1;
        }

        (Self(rem_nanos), seconds as i64)
    }

    /// ```
    /// # use timestamp::nanos::Nanos;
    /// // Since [`Nanos`] is at most 10^9, this will massively overflow.
    /// let (nanos, seconds) = Nanos::new_overflow_i64(i64::MAX);
    ///
    /// // Since i64::MAX = 9,223,372,036,854,775,807:
    ///
    /// assert_eq!(seconds, 9_223_372_036);
    /// assert_eq!(nanos.get(), 854_775_807);
    ///
    /// // Similarly, from a massive negative number:
    /// let (wrapped_nanos, neg_seconds) = Nanos::new_overflow_i64(i64::MIN);
    ///
    /// // Since i64::MIN = -9,223,372,036,854,775,808:
    ///
    /// assert_eq!(neg_seconds, -9_223_372_037);
    /// assert_eq!(wrapped_nanos.get(), 145_224_192);
    ///
    /// // since 'wrapped_nanos' needed to subtract a whole second to wrap back to a positive
    /// // number, we need to do pull that second out, otherwise we'll overflow converting
    /// // from seconds -> nanoseconds.
    ///
    /// let mut nanos = wrapped_nanos.get() as i64 - 1_000_000_000;
    /// // subtract the wrapped second (in nanoseconds) here ^
    /// nanos += (neg_seconds + 1) * 1_000_000_000;
    /// // and add it back here ^
    /// assert_eq!(nanos, i64::MIN);
    /// ```
    #[inline]
    pub const fn new_overflow_i64(nanos: i64) -> (Self, i64) {
        let mut seconds = nanos / conv::NANOS_PER_SECOND_I64;
        let mut rem_nanos = (nanos - (seconds * conv::NANOS_PER_SECOND_I64)) as i32;

        if rem_nanos.is_negative() {
            // take a second from here,
            seconds -= 1;
            // and add it here
            rem_nanos += conv::NANOS_PER_SECOND_I32;
        }

        // const-usable debug check to make sure that nanos is within range.
        if rem_nanos > MAX_NANOS || rem_nanos < 0 {
            #[allow(arithmetic_overflow)]
            let _ = i64::MAX + 1;
        }

        (Self(rem_nanos), seconds)
    }

    /// Gets the inner number of subsecond nanoseconds.
    #[inline]
    pub const fn get(&self) -> i32 {
        self.0
    }

    /// Builds a new [`Nanos`] with no checks on whether or not it's in the valid range of values.
    ///
    /// # Safety
    ///    - 'nanos' must be non-negative
    ///    - 'nanos' must be < '1_000_000_000'
    #[inline]
    pub const unsafe fn new_unchecked(nanos: i32) -> Self {
        Self(nanos)
    }

    /// ```
    /// # use timestamp::nanos::Nanos;
    /// let negative = Nanos::new_saturating(-1000);
    /// assert_eq!(negative.get(), 0);
    ///
    /// let over_1_sec = Nanos::new_saturating(2_000_000_000);
    /// assert_eq!(over_1_sec.get(), 999_999_999);
    /// ```
    pub const fn new_saturating(nanos: i32) -> Self {
        if nanos < 0 {
            Self(0)
        } else if nanos > MAX_NANOS {
            Self(MAX_NANOS)
        } else {
            Self(nanos)
        }
    }

    /// ```
    /// # use timestamp::nanos::Nanos;
    /// let negative = Nanos::new_wrapping(-1000);
    /// // an extra '+1' term is needed to account for 0 being a valid value.
    /// let wrapped = Nanos::new(Nanos::MAX.get() - 1000 + 1).unwrap();
    /// assert_eq!(negative, wrapped);
    ///
    /// let one_sec = Nanos::new_wrapping(1_000_000_000);
    /// assert_eq!(one_sec, Nanos::ZERO);
    ///
    /// let over_one_sec = Nanos::new_wrapping(1_000_000_001);
    /// let expected = Nanos::new(1).unwrap();
    /// assert_eq!(over_one_sec, expected);
    /// ```
    pub const fn new_wrapping(mut nanos: i32) -> Self {
        // TODO: replace with modulos instead of looping
        while nanos < 0 {
            nanos += conv::NANOS_PER_SECOND_I32;
        }

        while nanos > MAX_NANOS {
            nanos -= conv::NANOS_PER_SECOND_I32;
        }

        // let nanos = ((nanos - 1) + (MAX_NANOS + 1)) % (MAX_NANOS + 1);
        // let nanos = ((nanos % MOD_NANOS) + MOD_NANOS) % MOD_NANOS;

        Self(nanos)
    }

    /// returns 1 if equal or over 500_000_000 nanoseconds, 0 otherwise.
    pub const fn round(self) -> i64 {
        (self.0 >= HALF_SEC_NANOS) as i64
    }

    /// Adds 2 [`Nanos`], returning a tuple with the resulting subsecond nanoseconds, and the
    /// number of whole seconds from any overflows.
    pub const fn overflowing_add(self, rhs: Self) -> (Self, i64) {
        Self::new_overflow(self.0 + rhs.0)
    }

    /// Subtracts 2 [`Nanos`], returning a tuple with the resulting subsecond nanoseconds, and the
    /// number of whole seconds from any overflows
    pub const fn overflowing_sub(self, rhs: Self) -> (Self, i64) {
        Self::new_overflow(self.0 - rhs.0)
    }

    /// Multiplies a [`Nanos`] 'mul' times, returning a tuple with the resulting subsecond
    /// nanoseconds and the number of whole seconds from any overflows
    pub const fn overflowing_mul(self, mul: i32) -> (Self, i64) {
        if mul == 0 {
            return (Self::ZERO, 0);
        } else if mul == 1 {
            return (self, 0);
        }

        match self.0.checked_mul(mul) {
            Some(mul_nanos) => Self::new_overflow(mul_nanos),
            None => {
                // a const-usable debug check to verify that casting to i64 makes it impossible
                // to overflow when starting from within the valid range of an i32.
                //
                // This gets totally optimized out in release builds.
                let _ = (i32::MAX as i64) * (i32::MAX as i64);
                let wide_nanos = (mul as i64) * (self.0 as i64);
                Self::new_overflow_i64(wide_nanos)
            }
        }
    }

    /// Multiples a [`Nanos`] by a floating point multiplier, returning a tuple with the resulting
    /// subsecond nanoseconds and the number of whole seconds from any overflows
    pub const fn overflowing_mul_f64(self, mul: f64) -> (Self, i64) {
        let res = mul * self.0 as f64;
        Self::new_overflow(res as i32)
    }

    /// Divides a [`Nanos`] by an integer divisor, returning a tuple with the resulting
    /// subsecond nanoseconds and the number of whole seconds from any overflows (which should
    /// always be 0 in the case of division by an integer).
    pub const fn overflowing_div(self, div: i32) -> (Self, i64) {
        Self::new_overflow(self.0 / div)
    }

    /// Divides a [`Nanos`] by an floating point divisor, returning a tuple with the resulting
    /// subsecond nanoseconds and the number of whole seconds from any overflows
    pub const fn overflowing_div_f64(self, div: f64) -> (Self, i64) {
        let res = self.0 as f64 / div;
        Self::new_overflow(res as i32)
    }

    pub(crate) const fn const_cmp(self, other: Self) -> Ordering {
        if self.0 > other.0 {
            Ordering::Greater
        } else if self.0 < other.0 {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }

    /// Overflowing negation.
    /// ```
    /// # use timestamp::nanos::Nanos;
    /// let nanos = Nanos::new(250_000_000).unwrap();
    /// let (neg_nanos, seconds_offset) = nanos.overflowing_neg();
    /// assert_eq!(seconds_offset, -1);
    /// assert_eq!(neg_nanos.get(), 750_000_000);
    /// ```
    pub const fn overflowing_neg(self) -> (Self, i64) {
        if self.0 == 0 {
            (Self::ZERO, 0)
        } else {
            // SAFETY: NANOS_PER_SECOND subtracting a non-zero number of nanoseconds will always
            // return a value within 0..1 second.
            unsafe { (Self::new_unchecked(conv::NANOS_PER_SECOND_I32 - self.0), -1) }
        }
    }
}

impl PartialEq for Nanos {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<i32> for Nanos {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd for Nanos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<i32> for Nanos {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Ord for Nanos {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(feature = "rand")]
mod rand_impls {
    use super::{MAX_NANOS, Nanos};

    #[cfg(feature = "rand")]
    impl rand::distr::Distribution<Nanos> for rand::distr::StandardUniform {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Nanos {
            Nanos(rng.random_range(0..=MAX_NANOS))
        }
    }

    impl rand::distr::uniform::SampleUniform for Nanos {
        type Sampler = NanosSampler;
    }

    pub struct NanosSampler {
        min: i32,
        max: i32,
    }

    impl NanosSampler {
        fn new_from(min: Nanos, max: Nanos) -> Self {
            Self {
                max: max.0,
                min: min.0,
            }
        }
    }

    impl rand::distr::uniform::UniformSampler for NanosSampler {
        type X = Nanos;

        fn new<B1, B2>(low: B1, high: B2) -> Result<Self, rand::distr::uniform::Error>
        where
            B1: rand::distr::uniform::SampleBorrow<Self::X> + Sized,
            B2: rand::distr::uniform::SampleBorrow<Self::X> + Sized,
        {
            Ok(Self::new_from(*low.borrow(), *high.borrow()))
        }

        fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, rand::distr::uniform::Error>
        where
            B1: rand::distr::uniform::SampleBorrow<Self::X> + Sized,
            B2: rand::distr::uniform::SampleBorrow<Self::X> + Sized,
        {
            let high = Nanos::new_saturating(high.borrow().0 + 1);
            Ok(Self::new_from(*low.borrow(), high))
        }

        fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
            Nanos::new_saturating(rng.random_range(self.min..=self.max))
        }
    }
}
