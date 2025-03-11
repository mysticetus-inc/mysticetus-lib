use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::time::SystemTime;

use chrono::{DateTime, TimeZone};
use time::macros::datetime;

use crate::error::{ConvertError, Error, Num};
use crate::macros::{checked_cast, impl_math_ops};
use crate::nanos::Nanos;
use crate::parse::OnMissingTz;
use crate::{Date, Duration, Time, Unit, conv};

const HALF_SECOND_NANOS: i32 = 500_000_000;

/// A UTC timestamp, relative to the Unix Epoch.
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Copy, Eq, Hash)]
pub struct Timestamp {
    seconds: i64,
    nanos: Nanos,
}

impl PartialEq for Timestamp {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.seconds == other.seconds && self.nanos == other.nanos
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(other)
    }
}

#[cfg(feature = "deepsize")]
deepsize::known_deep_size!(0; Timestamp);

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Timestamp {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let seconds = u.int_in_range(Self::MIN.seconds..=Self::MAX.seconds)?;
        let nanos = Nanos::arbitrary(u)?;

        Ok(Self { seconds, nanos })
    }
}

/// Constants + common functions
impl Timestamp {
    /// The unix epoch.
    //    pub const UNIX_EPOCH: Self = Timestamp(Nanoseconds::from_nanos(0));
    pub const UNIX_EPOCH: Self = Self {
        seconds: 0,
        nanos: Nanos::ZERO,
    };

    /// The minimum representable timestamp. Corresponds to '0001-01-01 00:00:00 UTC'
    pub const MIN: Self = Self::from_offset_datetime(datetime!(0001-01-01 00:00:00).assume_utc());

    /// The maximally representable timestamp. Corresponds to '9999-12-31 23:59:59.999999999 UTC'
    pub const MAX: Self =
        Self::from_offset_datetime(datetime!(9999-12-31 23:59:59.999999999).assume_utc());

    // ------------------------------- Infallible Constructors -------------------------------- //

    /// Returns a timestamp representing the current system time. Will panic if the system time
    /// is set before the Unix Epoch, or later than the timestamp represented by
    /// [`Timestamp::MAX`].
    pub fn now() -> Self {
        let std_duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system time is before the Unix Epoch");

        let (seconds, nanos) = Duration::from(std_duration).into_parts();

        Self { seconds, nanos }
    }

    /// Rounds this timestamp to the nearest whole second.
    pub fn round_to_nearest_second(mut self) -> Self {
        if self.nanos >= HALF_SECOND_NANOS {
            self.seconds += 1;
        }
        self.nanos = Nanos::ZERO;
        self
    }

    /// Converts from a [`chrono::DateTime`]. Under the hood, the [`From`] impl uses this
    /// function.
    pub fn from_datetime<Tz>(datetime: DateTime<Tz>) -> Self
    where
        Tz: TimeZone,
    {
        let (nanos, secs) = Nanos::new_overflow(datetime.timestamp_subsec_nanos() as i32);

        Self {
            seconds: datetime.timestamp() + secs,
            nanos,
        }
    }

    /// Returns a [`time::Duration`] since the the unix epoch.
    #[inline]
    pub const fn duration_since_epoch(self) -> time::Duration {
        time::Duration::new(self.seconds, self.nanos.get())
    }

    /// Converts from a [`time::OffsetDateTime`]. Since all [`time::OffsetDateTime`] values are
    /// valid, this is infallible.
    pub const fn from_offset_datetime(dt: time::OffsetDateTime) -> Self {
        let (nanos, secs) = Nanos::new_overflow(dt.nanosecond() as i32);
        Self {
            seconds: dt.unix_timestamp() + secs,
            nanos,
        }
    }

    /// Constructs a timestamp from an integer number of nanoseconds since/before the unix epoch.
    ///
    /// ```
    /// # use timestamp::Timestamp;
    /// let nanos = 1234567;
    ///
    /// let time_offset_dt = time::OffsetDateTime::from_unix_timestamp_nanos(nanos as i128).unwrap();
    ///
    /// let timestamp = Timestamp::from_nanos(nanos);
    ///
    /// assert_eq!(timestamp, Timestamp::from_offset_datetime(time_offset_dt));
    /// ```
    pub const fn from_nanos(nanos: i64) -> Self {
        Self {
            seconds: nanos / super::conv::NANOS_PER_SECOND_I64,
            // SAFETY: constant is 1 second in nano seconds, so the modulo of that will always
            // be less than 1 full second.
            nanos: unsafe {
                Nanos::new_unchecked((nanos % super::conv::NANOS_PER_SECOND_I64) as i32)
            },
        }
    }

    /// Identical to [`Self::from_nanos`], but in [`i128`] which is less prone to overflows.
    /// If the date is out of range, this saturates and returns the maximum possible timestamp.
    pub const fn from_nanos_i128(nanos: i128) -> Self {
        match time::OffsetDateTime::from_unix_timestamp_nanos(nanos) {
            Ok(dt) => Self::from_offset_datetime(dt),
            Err(_) => Self::MAX,
        }
    }

    /// Constructs a timestamp from a floating point number of nanoseconds since/before the
    /// unix epoch. This does not check that the floating point number is in range, see
    /// [`Timestamp::from_nanos_f64_checked`] for a version that verifies the number is valid.
    pub const fn from_nanos_f64(nanos: f64) -> Self {
        Self::from_nanos(nanos as i64)
    }

    /// Converts from a unix timestamp, in integer milliseconds. Since this needs to convert to
    /// nanoseconds with the same integer type, overflowing is possible. This function
    /// performs no checks before converting. To use one that does check, see
    /// [`Timestamp::from_millis_checked`]
    pub const fn from_millis_saturating(millis: i64) -> Self {
        let (seconds, nanos) = Duration::from_millis_i64_saturating(millis).into_parts();
        Self { seconds, nanos }
    }

    /// Converts from a unix timestamp, in integer microseconds. Since this needs to convert to
    /// nanoseconds with the same integer type, overflowing is possible. This function
    /// performs no checks before converting. To use one that does check, see
    /// [`Timestamp::from_micros_checked`]
    pub const fn from_micros(micros: i64) -> Self {
        // Self(Nanoseconds::from_millis(millis))
        Self::from_nanos(micros * 1_000)
    }

    /// Converts from a unix timestamp, in floating point milliseconds. This function
    /// performs no checks before converting. To use one that does check, see
    /// [`Timestamp::from_millis_f64_checked`]
    pub const fn from_millis_f64(millis: f64) -> Self {
        //Self(Nanoseconds::from_millis_f64(millis))
        Self::from_nanos_f64(millis * 1e6)
    }

    /// Converts from a unix timestamp, in integer seconds. Since this needs to convert to
    /// nanoseconds with the same integer type, overflowing is very possible. This function
    /// performs no checks before converting. To use one that does check, see
    /// [`Timestamp::from_seconds_checked`]
    pub const fn from_seconds(seconds: i64) -> Self {
        // Self(Nanoseconds::from_seconds(seconds))
        Self {
            seconds,
            nanos: Nanos::ZERO,
        }
    }

    /// Converts from a unix timestamp, in floating point seconds. Does not check if the number of
    /// seconds fits within the range limit for timestamps. See
    /// [`Timestamp::from_seconds_f64_checked`] for the safe version that checks that the number of
    /// seconds is within range.
    pub const fn from_seconds_f64(seconds: f64) -> Self {
        Self::from_nanos_f64(seconds * conv::NANOS_PER_SECOND_F64)
    }

    /// Converts from MS ticks. Conversion pulled from the [`DateTimeOffset.ToUnixTimeSeconds`]
    /// source code (after converting from seconds -> nanoseconds to keep precision). This version
    /// of the function does not check that the timestamp in ticks lies within the valid range of
    /// [`Timestamp`]'s. See [`Timestamp::from_ticks_checked`] for a checked version.
    ///
    /// [`DateTimeOffset.ToUnixTimeSeconds`]: <https://referencesource.microsoft.com/#mscorlib/system/datetimeoffset.cs,8e1e87bf153c720e>
    pub const fn from_ticks(ticks: u64) -> Self {
        // nanoseconds since the beginning of the ticks epoch. Cast to i128 so we have room to
        // work with as we multiply/offset
        let raw_nanos = ticks as i128 * conv::NANOS_PER_TICK as i128;

        // offset to begin at the beginning of the unix epoch
        let nanos_unix = raw_nanos - conv::TICKS_UNIX_EPOCH_OFFSET_NANOS;

        // cast to i64 now that we're supposedly within the range of valid values
        // Self(Nanoseconds::from_nanos(nanos_unix as i64))
        Self::from_nanos(nanos_unix as i64)
    }

    /// Convers from floating point MS ticks. Does a simple `as` cast to [`u64`], then defers to
    /// [`Timestamp::from_ticks`]. See [`Timestamp::from_ticks_f64_checked`] for a version
    /// that verifies that the floating point value is valid for a [`Timestamp`]
    pub const fn from_ticks_f64(ticks: f64) -> Self {
        Self::from_ticks(ticks as u64)
    }

    /// Adds a duration in a fully checked manner.
    pub const fn add_duration_checked(self, duration: Duration) -> Option<Self> {
        if let Ok(dt) = self.as_offset_datetime()
            && let Some(checked) = dt.checked_add(duration.into_time_duration())
        {
            return Some(Self::from_offset_datetime(checked));
        }

        None
    }

    /// Attempts to add the given [`Duration`], and saturates at [`Timestamp::MIN`]
    /// and [`Timestamp::MAX`] if the value would overflow.
    pub const fn add_duration_saturating(self, duration: Duration) -> Self {
        match self.add_duration_checked(duration) {
            Some(ok) => ok,
            None if duration.is_positive() => Timestamp::MAX,
            None => Timestamp::MIN,
        }
    }

    /// Subtracts a duration in a fully checked manner.
    pub const fn sub_duration_checked(self, duration: Duration) -> Option<Self> {
        if let Ok(dt) = self.as_offset_datetime()
            && let Some(checked) = dt.checked_sub(duration.into_time_duration())
        {
            return Some(Self::from_offset_datetime(checked));
        }

        None
    }

    /// Adds a [`Duration`]. Used by the [`std::ops::Add`] impl under the hood.
    #[inline]
    pub const fn add_duration(self, duration: Duration) -> Self {
        let (seconds, nanos) = Duration::from_parts(self.seconds, self.nanos)
            .saturating_add(duration)
            .into_parts();

        Self { seconds, nanos }
    }

    // ------------------------------- Fallible Constructors ---------------------------------- //

    /// Attempts to parse a string with a datetime format. This defers to the [`FromStr`] impl on
    /// [`chrono::DateTime`]. To specify a format, see [`Timestamp::from_datetime_str_with_fmt`].
    ///
    /// The [`FromStr`] impl on [`Timestamp`] calls this under the hood.
    ///
    /// Identical to calling [`Timestamp::from_datetime_str_opt`], with 'on_missing_tz' set to
    /// the default, [`OnMissingTz::Warn`].
    pub fn from_datetime_str<S>(dt: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Self::from_datetime_str_opt(dt.as_ref(), OnMissingTz::Warn)
    }

    /// Parses a timestamp string, assuming UTC if no timezone is specified.
    ///
    /// Identical to calling [`Timestamp::from_datetime_str_opt`] with 'on_missing_tz' set to
    /// [`OnMissingTz::Ignore`].
    pub fn from_datetime_str_assume_utc<S>(dt: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Self::from_datetime_str_opt(dt.as_ref(), OnMissingTz::Ignore)
    }

    /// Attempts to parse a string with a datetime format. This defers to the [`FromStr`] impl on
    /// [`chrono::DateTime`]. To specify a format, see [`Timestamp::from_datetime_str_with_fmt`].
    ///
    /// The [`FromStr`] impl on [`Timestamp`] calls this under the hood.
    pub fn from_datetime_str_opt<S>(dt: S, on_missing_tz: OnMissingTz) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        fn inner(dt: &str, on_missing_tz: OnMissingTz) -> Result<Timestamp, Error> {
            crate::parse::parse_timestamp(dt, on_missing_tz).map_err(Error::ParseParts)
        }

        inner(dt.as_ref(), on_missing_tz)
    }

    /// Parses a datetime string with a given format. Defers to
    /// [`chrono::Utc::datetime_from_str`] under the hood.
    pub fn from_datetime_str_with_fmt<S, F>(dt: S, fmt: F) -> Result<Self, Error>
    where
        S: AsRef<str>,
        F: AsRef<str>,
    {
        let dt = chrono::DateTime::parse_from_str(dt.as_ref(), fmt.as_ref())?;
        Ok(Self::from_datetime(dt))
    }

    /// Builds a [`Timestamp`] from a number of nanoseconds since the unix epoch (as [`i128`]).
    /// Checks to make sure the resulting timestamp is within the range of valid values.
    pub const fn from_nanos_i128_checked(nanos: i128) -> Result<Self, ConvertError> {
        match time::OffsetDateTime::from_unix_timestamp_nanos(nanos) {
            Ok(dt) => Ok(Self::from_offset_datetime(dt)),
            Err(_) => Err(ConvertError::OutOfRange),
        }
    }

    /// Casts to microseconds.
    pub const fn as_micros(&self) -> i64 {
        let micros = self.as_nanos() / 1000;
        micros as i64
    }

    /// Casts to microseconds, in [`f64`].
    pub const fn as_micros_f64(&self) -> f64 {
        self.as_nanos_f64() / 1000.0
    }

    /// Safe version of [`Timestamp::from_nanos_f64`] that checks that the floating point value
    /// is valid in context. If there is an error, returns [`ConvertError`] as the
    /// [`Err`] variant.
    pub const fn from_nanos_f64_checked(nanos: f64) -> Result<Self, ConvertError> {
        match checked_cast!(nanos; f64[F64] => i128[I128]) {
            Ok(nanos) => Self::from_nanos_i128_checked(nanos),
            Err(err) => Err(err),
        }

        /*
        let seconds = nanos / conv::NANOS_PER_SECOND_F64;

        if !seconds.is_finite() {
            return Err(ConvertError::overflow_unit(Some(Unit::Millis), Unit::Seconds, Num::F64));
        }

        let dur = time::Duration::seconds_f64(seconds);

        Ok(Self(protos::protobuf::Timestamp {
            seconds: dur.whole_seconds(),
            nanos: dur.subsec_nanoseconds(),
        }))
        */
    }

    /// Converts from milliseconds, but checks for overflow when converting into the internal
    /// representation in nanoseconds. If there is an overflow, returns [`ConvertError`] as the
    /// [`Err`] variant.
    pub const fn from_millis_checked(millis: i64) -> Result<Self, ConvertError> {
        let (seconds, nanos) = match Duration::from_millis_i64_checked(millis) {
            Some(dur) => dur.into_parts(),
            None => return Err(ConvertError::OutOfRange),
        };

        Ok(Self { seconds, nanos })
    }

    /// Converts from microseconds, but checks for overflow when converting into the internal
    /// representation in nanoseconds. If there is an overflow, returns [`ConvertError`] as the
    /// [`Err`] variant.
    pub const fn from_micros_checked(micros: i64) -> Result<Self, ConvertError> {
        let dur = time::Duration::microseconds(micros);

        match time::OffsetDateTime::UNIX_EPOCH.checked_add(dur) {
            Some(offset) => Ok(Self::from_offset_datetime(offset)),
            None => Err(ConvertError::OutOfRange),
        }
    }

    /// Converts from microseconds, but checks for overflow when converting into the internal
    /// representation in nanoseconds. If there is an overflow, returns [`ConvertError`] as the
    /// [`Err`] variant.
    pub const fn from_micros_f64_checked(micros: f64) -> Result<Self, ConvertError> {
        Self::from_nanos_f64_checked(micros * 1000.0)
    }

    /// Converts from milliseconds, but checks that the resulting internal representation is
    /// valid. If there is an overflow, returns [`ConvertError`] as the [`Err`] variant.
    pub const fn from_millis_f64_checked(millis: f64) -> Result<Self, ConvertError> {
        Self::from_seconds_f64_checked(millis / conv::MILLIS_PER_SECOND_F64)
    }

    /// Converts from seconds, but checks for overflow when converting into the internal
    /// representation in nanoseconds. If there is an overflow, returns [`ConvertError`] as the
    /// [`Err`] variant.
    pub const fn from_seconds_checked(seconds: i64) -> Result<Self, ConvertError> {
        match time::OffsetDateTime::from_unix_timestamp(seconds) {
            Ok(dt) => Ok(Self::from_offset_datetime(dt)),
            Err(_) => Err(ConvertError::OutOfRange),
        }
    }

    /// Attempts to convert from a floating point number of seconds from the unix epoch, checking
    /// that the number of seconds is within the range of [`Timestamp::MIN`] and
    /// [`Timestamp::MAX`]
    pub const fn from_seconds_f64_checked(seconds: f64) -> Result<Self, ConvertError> {
        let nanos = seconds * conv::NANOS_PER_SECOND_F64;

        if !nanos.is_finite() {
            return Err(ConvertError::NotFinite);
        }

        match time::OffsetDateTime::from_unix_timestamp_nanos(nanos as i128) {
            Ok(dt) => Ok(Self::from_offset_datetime(dt)),
            Err(_) => Err(ConvertError::OutOfRange),
        }
    }

    /// Safer version of [`Timestamp::from_ticks`] that checks for integer overflow/underflow
    /// when converting from ticks to the underlying nanosecond container.
    pub const fn from_ticks_checked(ticks: u128) -> Result<Self, ConvertError> {
        let raw_nanos = match ticks.checked_mul(conv::NANOS_PER_TICK as u128) {
            Some(nanos) => nanos,
            _ => return Err(ConvertError::overflow_unit(None, Unit::Nanos, Num::U64)),
        };

        // the offset to the unix epoch may require going negative, so convert to i128 here.
        let signed_nanos: i128 = match checked_cast!(raw_nanos; u128[U128] => i128[I128]) {
            Ok(nanos) => nanos,
            Err(err) => return Err(err),
        };

        let nanos_unix = match signed_nanos.checked_sub(conv::TICKS_UNIX_EPOCH_OFFSET_NANOS) {
            Some(nanos_unix) => nanos_unix,
            _ => {
                return Err(ConvertError::overflow_unit(
                    Some(Unit::Nanos),
                    Unit::Nanos,
                    Num::U64,
                ));
            }
        };

        Self::from_nanos_i128_checked(nanos_unix)
    }

    /// Safer version of [`Timestamp::from_ticks_f64`] that checks and makes sure that the
    /// floating point value can be coerced into a valid timestamp in nanoseconds.
    pub const fn from_ticks_f64_checked(ticks: f64) -> Result<Self, ConvertError> {
        match checked_cast!(ticks; f64[F64] => u128[U128]) {
            Ok(casted) => Self::from_ticks_checked(casted),
            Err(err) => Err(err),
        }
    }

    // ----------------------------------- Conversions -------------------------------------- //

    /// Attempts to convert to a [`time::OffsetDateTime`]
    pub const fn as_offset_datetime(&self) -> Result<time::OffsetDateTime, ConvertError> {
        match time::OffsetDateTime::from_unix_timestamp_nanos(self.as_nanos()) {
            Ok(dt) => Ok(dt),
            Err(_) => Err(ConvertError::OutOfRange),
        }
    }

    /// Attempts to convert to a [`time::PrimitiveDateTime`]
    pub const fn as_primitive_datetime(&self) -> Result<time::PrimitiveDateTime, ConvertError> {
        match self.as_offset_datetime() {
            Ok(dt) => Ok(time::PrimitiveDateTime::new(dt.date(), dt.time())),
            Err(_) => Err(ConvertError::OutOfRange),
        }
    }

    /// Attempts to convert to a [`Date`], saturating at [`Date::MAX`] if out of range.
    pub const fn date(&self) -> Date {
        match time::OffsetDateTime::from_unix_timestamp_nanos(self.as_nanos()) {
            Ok(offset_dt) => Date::from_time(offset_dt.date()),
            _ => Date::MAX,
        }
    }

    /// Attempts to convert to a [`Time`], saturating at [`Time::MAX`] if out of range.
    pub const fn time(&self) -> Time {
        match time::OffsetDateTime::from_unix_timestamp_nanos(self.as_nanos()) {
            Ok(dt) => Time::from_time(dt.time()),
            Err(_) => Time::MAX,
        }
    }

    /// Converts to a [`chrono::DateTime<chrono::Utc>`] instance.
    pub fn as_datetime(&self) -> DateTime<chrono::Utc> {
        self.as_datetime_with_tz(chrono::Utc)
    }

    /// Builds an ISO8601 valid string from the [`Timestamp`]. Under the hood, this only returns
    /// RFC3339 strings, which is a subset of ISO8601. Used by the [`std::fmt::Display`] impl.
    pub fn as_iso8601(&self) -> String {
        self.as_datetime()
            .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
    }

    /// Appends the ISO8601 repr to a provided string buffer.
    pub fn append_iso8601(&self, dst: &mut String, frac_digits: usize) {
        self.date().append_to_string(dst);
        dst.push('T');
        self.time().append_to_string(dst, frac_digits);
    }

    /// Converts to ISO8601, but with a ' ' separator instead of 'T'. Truncates all fractional
    /// seconds.
    pub fn as_space_separated_iso8601(&self) -> String {
        let dt = self
            .as_primitive_datetime()
            .unwrap_or(time::PrimitiveDateTime::MAX);
        let (year, month, day) = dt.date().to_calendar_date();
        let (hrs, mins, secs) = dt.time().as_hms();

        format!("{year}-{month}-{day} {hrs:02}:{mins:02}:{secs:02}Z")
    }

    /// Convert to an arbitrary unit (as a signed int).
    pub const fn to_unit_signed(&self, unit: Unit) -> i64 {
        match unit {
            Unit::Seconds => self.as_seconds(),
            Unit::Nanos => self.as_nanos() as i64,
            Unit::Micros => self.as_micros(),
            Unit::Millis => self.as_millis(),
        }
    }

    /// Convert to an arbitrary unit (as an unsigned int). Returns [`Err`] if the inner signed
    /// integer is negative.
    pub const fn to_unit_unsigned(&self, unit: Unit) -> Result<u64, ConvertError> {
        let signed_int = match unit {
            Unit::Seconds => self.as_seconds(),
            Unit::Millis => self.as_millis(),
            Unit::Micros => self.as_micros(),
            Unit::Nanos => match checked_cast!(self.as_nanos(); i128[I128] => i64[I64]) {
                Ok(nanos) => nanos,
                Err(err) => return Err(err),
            },
        };

        match checked_cast!(signed_int; i64[I64] => u64[U64]) {
            Ok(uint) => Ok(uint),
            Err(err) => Err(err),
        }
    }

    /// Convert to an arbitrary unit, as a floating point number.
    pub const fn to_unit_f64(&self, unit: Unit) -> f64 {
        match unit {
            Unit::Seconds => self.as_seconds_f64(),
            Unit::Nanos => self.as_nanos_f64(),
            Unit::Micros => self.as_micros_f64(),
            Unit::Millis => self.as_millis_f64(),
        }
    }

    /// Converts to a [`chrono::DateTime`] instance with any valid [`TimeZone`].
    pub fn as_datetime_with_tz<Tz>(&self, timezone: Tz) -> DateTime<Tz>
    where
        Tz: TimeZone,
    {
        timezone
            .timestamp_opt(self.seconds, self.nanos.get() as u32)
            .unwrap()
    }

    /// Returns this timestamp as the number of integer seconds since/before the unix epoch.
    pub const fn as_seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns this timestamp as the number of floating point seconds since/before the unix epoch.
    ///
    /// ```
    /// # use timestamp::Timestamp;
    /// let seconds = 1234532.46;
    ///
    /// let timestamp = Timestamp::from_seconds_f64(seconds);
    /// println!("{timestamp:#?} - {}", timestamp.as_seconds_f64());
    /// let delta = (timestamp.as_seconds_f64() - seconds).abs();
    /// assert!(delta < 1e-9);
    /// ```
    pub const fn as_seconds_f64(&self) -> f64 {
        (self.seconds as f64) + (self.nanos.get() as f64 / conv::NANOS_PER_SECOND_F64)
    }

    /// Returns this timestamp as the number of integer milliseconds since/before the unix epoch.
    pub const fn as_millis(&self) -> i64 {
        (self.seconds * conv::MILLIS_PER_SECOND) + (self.nanos.get() as i64 / conv::NANOS_PER_MILLI)
    }

    /// Returns this timestamp as the number of floating point milliseconds since the unix epoch.
    pub const fn as_millis_f64(&self) -> f64 {
        (self.seconds as f64 * conv::MILLIS_PER_SECOND_F64)
            + (self.nanos.get() as f64 / conv::NANOS_PER_MILLI_F64)
    }

    /// Returns this timestamp as the number of integer nanoseconds since/before the unix epoch.
    pub const fn as_nanos(&self) -> i128 {
        (self.seconds as i128 * conv::NANOS_PER_SECOND_I128) + self.nanos.get() as i128
    }

    /// Returns this timestamp as the number of integer nanoseconds since/before the unix epoch.
    pub const fn as_nanos_f64(&self) -> f64 {
        self.as_nanos() as f64
    }

    /// Converts the underlying timestamp to MS Ticks. This function does not check whether or not
    /// the underlying math overflows in any way. To use a safer function that does check for
    /// both data type overflows + unit conversion overflows, use [`as_ticks_checked`].
    ///
    /// [`as_ticks_checked`]: [`Self::as_ticks_checked`]
    pub const fn as_ticks(&self) -> u64 {
        let raw_nanos = self.as_nanos();

        // add the offset to go from being based on the unix epoch to the ticks epoch
        let offset_nanos = raw_nanos + conv::TICKS_UNIX_EPOCH_OFFSET_NANOS;

        // then convert to ticks (100ns = 1 tick)
        (offset_nanos / conv::NANOS_PER_TICK as i128) as u64
    }

    /// Converts to MS Ticks. If any of the math causes an overflow, the [`Err`] variant is
    /// returned.
    pub const fn as_ticks_checked(&self) -> Result<u64, ConvertError> {
        let raw_nanos = self.as_nanos();

        // add the offset to go from being based on the unix epoch to the ticks epoch
        let offset_nanos = match raw_nanos.checked_add(conv::TICKS_UNIX_EPOCH_OFFSET_NANOS) {
            Some(offset) => offset,
            _ => {
                return Err(ConvertError::overflow_unit(
                    Some(Unit::Nanos),
                    Unit::Nanos,
                    Num::I128,
                ));
            }
        };

        // then convert to ticks (100ns = 1 tick)
        Ok((offset_nanos / conv::NANOS_PER_TICK as i128) as u64)
    }

    /// Simple conveinence function that takes the result of [`as_ticks`] and casts it to
    /// an [`f64`].
    ///
    /// [`as_ticks`]: [`Self::as_ticks`]
    pub const fn as_ticks_f64(&self) -> f64 {
        self.as_ticks() as f64
    }

    /// Returns the number of sub-second nanoseconds.
    pub const fn subsec_nanos(&self) -> i32 {
        self.nanos.get()
    }

    // -------------------------------- Offset/Add Methods ---------------------------------- //

    /// Adds a given number of nanoseconds. Does not check for overflows.
    pub const fn add_nanos(self, nanos: i64) -> Self {
        self.add_duration(Duration::from_nanos(nanos as _))
    }

    /// Adds a floating point number of nanoseconds. Does not check for overflows.
    pub const fn add_nanos_f64(self, nanos: f64) -> Self {
        self.add_nanos(nanos as i64)
    }

    /// Adds a whole number of milliseconds. Does not check for overflows.
    pub const fn add_millis(self, millis: i64) -> Self {
        self.add_nanos(millis * crate::conv::NANOS_PER_MILLI)
    }

    /// Adds a floating point number of milliseconds. Does not check for overflows.
    pub const fn add_millis_f64(self, millis: f64) -> Self {
        self.add_nanos_f64(millis * 1e6)
    }

    /// Adds a whole number of seconds. Does not check for overflows.
    pub const fn add_seconds(self, seconds: i64) -> Self {
        Self {
            seconds: self.seconds + seconds,
            nanos: self.nanos,
        }
    }

    /// Adds a floating point number of seconds. Does not check for overflows.
    pub const fn add_seconds_f64(self, secs: f64) -> Self {
        self.add_nanos_f64(secs * conv::NANOS_PER_SECOND_F64)
    }

    /// Adds a [`time::Duration`] and returns the resulting timestamp. Uses the [`std::ops::Add`]
    /// impl under the hood.
    pub const fn add_time_duration(self, duration: time::Duration) -> Self {
        self.add_duration(Duration::from_time_duration(duration))
    }

    /// Adds a [`std::time::Duration`] and returns the resulting timestamp. Used by the
    /// [`std::ops::Add`] impl under the hood.
    pub fn add_std_duration(self, duration: std::time::Duration) -> Self {
        // let dur = time::Duration::new(duration.as_secs() as i64, duration.subsec_nanos() as i32);
        self.add_duration(duration.into())
    }

    /// Subtracts a [`std::time::Duration`] and returns the resulting timestamp. Used by the
    /// [`std::ops::Sub`] impl under the hood.
    pub const fn sub_std_duration(self, duration: std::time::Duration) -> Self {
        let secs = duration.as_secs() as i64;
        let nanos = duration.subsec_nanos() as i32;

        self.add_duration(Duration::new(-secs, -nanos))
    }

    /// Subtracts the two [`Timestamp`], getting the delta between the two.
    pub const fn delta(self, other: Self) -> Duration {
        let seconds = self.seconds - other.seconds;
        let (nanos, overflow) = self.nanos.overflowing_sub(other.nanos);

        Duration::from_parts(seconds + overflow, nanos)
    }

    /// const-able [`Ord::cmp`].
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

impl FromStr for Timestamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().all(|c| matches!(c, '0'..='9' | '.')) {
            match s.parse::<f64>() {
                Ok(float) => Timestamp::from_seconds_f64_checked(float).map_err(Error::Convert),
                Err(error) => Err(Error::FloatParse(error)),
            }
        } else {
            Self::from_datetime_str(s)
        }
    }
}

impl<Tz> From<DateTime<Tz>> for Timestamp
where
    Tz: TimeZone,
{
    fn from(dt: DateTime<Tz>) -> Self {
        Self::from_datetime(dt)
    }
}

impl From<time::OffsetDateTime> for Timestamp {
    fn from(dt: time::OffsetDateTime) -> Self {
        Self::from_offset_datetime(dt)
    }
}

impl From<Timestamp> for time::OffsetDateTime {
    fn from(ts: Timestamp) -> Self {
        match time::OffsetDateTime::from_unix_timestamp_nanos(ts.as_nanos()) {
            Ok(offset_dt) => offset_dt,
            Err(_) => panic!("out of range for time::OffsetDateTime"),
        }
    }
}

impl_math_ops! {
    Timestamp => {
        Add => (Output = Self) => add(self: Self, rhs: Duration) -> Self {
            self.add_duration(rhs)
        },
        AddAssign => add_assign(self: &mut Self, rhs: Duration) { *self = *self + rhs; },
        Add => (Output = Self) => add(self: Self, rhs: time::Duration) -> Self {
            self.add_time_duration(rhs)
        },
        AddAssign => add_assign(self: &mut Self, rhs: time::Duration) { *self = *self + rhs; },
        Add => (Output = Self) => add(self: Self, rhs: std::time::Duration) -> Self {
            self.add_std_duration(rhs)
        },
        AddAssign => add_assign(self: &mut Self, rhs: std::time::Duration) {
            *self = *self + rhs;
        },
        Sub => (Output = Self) => sub(self: Self, rhs: time::Duration) -> Self {
            self.add_time_duration(-rhs)
        },
        SubAssign => sub_assign(self: &mut Self, rhs: time::Duration) { *self = *self - rhs; },
        Sub => (Output = Self) => sub(self: Self, rhs: std::time::Duration) -> Self {
            self.sub_std_duration(rhs)
        },
        SubAssign => sub_assign(self: &mut Self, rhs: std::time::Duration) {
            *self = *self - rhs;
        },
        Sub => (Output = Self) => sub(self: Self, rhs: crate::Duration) -> Self {
            self.add_duration(-rhs)
        },
        SubAssign => sub_assign(self: &mut Self, rhs: crate::Duration) {
            *self = *self - rhs;
        },
        Sub => (Output = Duration) => sub(self: Self, rhs: Self) -> Duration {
            self.delta(rhs)
        },
    }
}

impl<Tz> PartialEq<DateTime<Tz>> for Timestamp
where
    Tz: TimeZone,
{
    fn eq(&self, other: &DateTime<Tz>) -> bool {
        self.seconds == other.timestamp()
            && self.nanos.get() == other.timestamp_subsec_nanos() as i32
    }
}

impl<Tz> PartialOrd<DateTime<Tz>> for Timestamp
where
    Tz: TimeZone,
{
    fn partial_cmp(&self, other: &DateTime<Tz>) -> Option<std::cmp::Ordering> {
        match self.seconds.cmp(&other.timestamp()) {
            std::cmp::Ordering::Equal => Some(
                self.nanos
                    .get()
                    .cmp(&(other.timestamp_subsec_nanos() as i32)),
            ),
            other_ord => Some(other_ord),
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // Treat the timestamp as a transparent wrapper, showing the nanos/seconds in the
        // underlying time::Duration struct.
        formatter
            .debug_struct("Timestamp")
            .field("seconds", &self.seconds)
            .field("nanos", &self.nanos)
            .finish()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_iso8601())
    }
}

impl TryFrom<serde_json::Value> for Timestamp {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

#[cfg(feature = "prost")]
impl From<prost_types::Timestamp> for Timestamp {
    fn from(ts: prost_types::Timestamp) -> Self {
        let (nanos, overflow) = Nanos::new_overflow(ts.nanos);
        Self {
            seconds: ts.seconds + overflow,
            nanos,
        }
    }
}

#[cfg(feature = "prost")]
impl From<Timestamp> for prost_types::Timestamp {
    fn from(ts: Timestamp) -> Self {
        Self {
            seconds: ts.seconds,
            nanos: ts.nanos.get(),
        }
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for Timestamp {
    fn schema_name() -> String {
        "Timestamp".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        chrono::DateTime::<chrono::Utc>::json_schema(gen)
    }
}

#[cfg(feature = "rand")]
mod rand_impls {
    use super::Timestamp;
    use crate::Duration;

    #[cfg(feature = "rand")]
    impl rand::distr::Distribution<Timestamp> for rand::distr::StandardUniform {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Timestamp {
            Timestamp {
                seconds: rng.random_range(Timestamp::MIN.seconds..=Timestamp::MAX.seconds),
                nanos: rng.random(),
            }
        }
    }

    impl rand::distr::uniform::SampleUniform for Timestamp {
        type Sampler = TimestampSampler;
    }

    pub struct TimestampSampler {
        start_at: Timestamp,
        delta: Duration,
    }

    impl TimestampSampler {
        fn new_from(start_at: Timestamp, end_before: Timestamp) -> Self {
            Self {
                delta: end_before - start_at,
                start_at,
            }
        }
    }

    impl rand::distr::uniform::UniformSampler for TimestampSampler {
        type X = Timestamp;

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
            Ok(Self::new_from(*low.borrow(), high.borrow().add_nanos(1)))
        }

        fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
            let delta = rng.random_range(Duration::ZERO..self.delta);
            self.start_at + delta
        }
    }
}
