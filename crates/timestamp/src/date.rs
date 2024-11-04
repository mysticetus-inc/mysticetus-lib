//! [`Date`] and assiciated impls.
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::iter::Step;
use std::num::NonZeroU8;
use std::str::FromStr;
use std::{fmt, ops};

use chrono::Datelike;
use serde::{Deserialize, Serialize};

use super::{Month, Time, Timestamp};

#[doc(hidden)]
pub const MIN_YEAR: i16 = -9999;
#[doc(hidden)]
pub const MAX_YEAR: i16 = 9999;

/// A date, between '-9999-01-01' and '9999-12-31'.
#[derive(Clone, Copy)]
pub struct Date {
    year: i16,
    month: Month,
    day: NonZeroU8,
}

#[cfg(feature = "deepsize")]
deepsize::known_deep_size!(0; Date);

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Date {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let year = u.int_in_range(-9999..=9999)?;
        let month: u8 = u.int_in_range(1..=12)?;

        let month = Month::from_number(month).expect("1..=12 is valid");

        let max_day = month.days_in(year);
        let day = u.int_in_range(1..=max_day)?;
        // SAFETY: we get the int within the range 1..#, so this is always non-zero
        let day = unsafe { NonZeroU8::new_unchecked(day) };

        Ok(Self { year, month, day })
    }
}

impl PartialEq for Date {
    fn eq(&self, rhs: &Self) -> bool {
        self.year == rhs.year && self.month == rhs.month && self.day.get() == rhs.day.get()
    }
}

impl Eq for Date {}

impl Hash for Date {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i16(self.year);
        state.write_u8(self.month as u8);
        state.write_u8(self.day.get());
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(other)
    }
}

/// Constructs a [`Date`] at compile time, using the [`DateBuilder`] helper.
#[macro_export]
macro_rules! date {
    ($year:literal - $month:literal - $day:literal) => {{
        let year: i16 = $year;
        let month: u8 = $month;
        let day: u8 = $day;

        $crate::Date::builder()
            .year(year)
            .month_digit(month)
            .day(day)
    }};
}

#[test]
fn test_date_macro() {
    assert_eq!(date!(-9999 - 1 - 1), crate::Date::MIN);
    assert_eq!(date!(9999 - 12 - 31), crate::Date::MAX);
}

impl fmt::Debug for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Date")
            .field("year", &self.year)
            .field("month", &self.month)
            .field("day", &self.day.get())
            .finish()
    }
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format_into(formatter)
    }
}

/// Builder for a [`Date`]. Can be used in const-contexts if none of the arguments need to be
/// parsed/etc.
#[derive(Default)]
pub struct DateBuilder<Y = (), M = (), D = ()> {
    year: Y,
    month: M,
    day: D,
}

impl DateBuilder {
    /// Initializes an empty [`DateBuilder`]. Only exists because 'const' [`Default`] was removed.
    #[inline]
    pub const fn new() -> Self {
        Self {
            year: (),
            month: (),
            day: (),
        }
    }
}

impl DateBuilder<(), (), ()> {
    /// Get and use the current year.
    #[inline]
    pub fn this_year(self) -> DateBuilder<i16, (), ()> {
        self.year(chrono::Utc::now().year() as i16)
    }

    /// Use the given year, checking to make sure it's within '-9999..=9999'. If outside
    /// that range, this returns [`None`].
    #[inline]
    pub const fn year_checked(self, year: i16) -> Option<DateBuilder<i16, (), ()>> {
        if year < -9999 || 9999 < year {
            return None;
        }

        Some(DateBuilder {
            year,
            month: self.month,
            day: self.day,
        })
    }

    /// Uses the given year. OPanics if the year is out of range ('-9999..=9999'). See
    /// [`year_checked`] for a non-panicing variant.
    ///
    /// [`year_checked`]: [`DateBuilder::year_checked`]
    #[inline]
    pub const fn year(self, year: i16) -> DateBuilder<i16, (), ()> {
        match self.year_checked(year) {
            Some(ok) => ok,
            None => panic!("year out of range, must be within '-9999..=9999'"),
        }
    }
}

macro_rules! impl_month_fns {
    ($($fn_name:ident:$month:ident),* $(,)?) => {
        $(
            #[doc = concat!(" Identical to calling `builder.month(Month::", stringify!($month), ")`.")]
            #[inline]
            pub const fn $fn_name(self) -> DateBuilder<i16, $crate::Month, ()> {
                self.month($crate::Month::$month)
            }
        )*
    };
}

impl DateBuilder<i16, (), ()> {
    /// Constructs the [`Date`] with the given month.
    #[inline]
    pub const fn month(self, month: Month) -> DateBuilder<i16, Month, ()> {
        DateBuilder {
            year: self.year,
            month,
            day: self.day,
        }
    }

    /// Converts the digit 1..=12 into a month, and uses the given month. If the month is outside
    /// that range, returns [`None`]. Checked, non-panicking variant of
    /// [`DateBuilder::month_digit`].
    pub const fn month_digit_checked(self, month: u8) -> Option<DateBuilder<i16, Month, ()>> {
        match Month::from_number(month) {
            Some(month) => Some(self.month(month)),
            None => None,
        }
    }

    /// Converts a month from a digit, where 1 = January, and so on.
    pub const fn month_digit(self, month: u8) -> DateBuilder<i16, Month, ()> {
        assert!(0 < month && month < 13);
        // SAFETY: assertion makes sure the value is a valid variant of `Month`
        let month = unsafe { std::mem::transmute::<u8, Month>(month) };
        self.month(month)
    }

    impl_month_fns! {
        january: January,
        feburary: February,
        march: March,
        april: April,
        may: May,
        june: June,
        july: July,
        august: August,
        september: September,
        october: October,
        november: November,
        december: December,
    }
}

impl DateBuilder<i16, Month, ()> {
    /// Constructs the [`Date`], checking to make sure the day is valid for the already
    /// specified month and year.
    ///
    /// This is the checked variant for [`DateBuilder::day`].
    pub const fn day_checked(self, day: u8) -> Option<Date> {
        let max = self.month.days_in(self.year);

        if day > max {
            return None;
        }

        let day = match NonZeroU8::new(day) {
            Some(day) => day,
            None => return None,
        };

        Some(Date {
            year: self.year,
            month: self.month,
            day,
        })
    }

    /// Constructs the [`Date`] from the given day.
    ///
    /// Panics if the day is out of range for the already specified
    /// month + year. See [`DateBuilder::day_checked`] for a checked,
    /// non-panicking variant.
    pub const fn day(self, day: u8) -> Date {
        match self.day_checked(day) {
            Some(date) => date,
            None => panic!("day out of range for the given month/day"),
        }
    }
}

/// Information about an invalid date.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "deepsize", derive(deepsize::DeepSizeOf))]
pub struct InvalidDate {
    field: DateField,
    reason: InvalidDateReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "deepsize", derive(deepsize::DeepSizeOf))]
enum DateField {
    Month,
    Year,
    Day,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "deepsize", derive(deepsize::DeepSizeOf))]
enum InvalidDateReason {
    OutOfRange,
    Missing,
    Invalid,
}

impl fmt::Display for InvalidDate {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let field = match self.field {
            DateField::Day => "day",
            DateField::Month => "month",
            DateField::Year => "year",
        };

        let reason = match self.reason {
            InvalidDateReason::OutOfRange => "out of range",
            InvalidDateReason::Missing => "missing",
            InvalidDateReason::Invalid => "invalid",
        };

        write!(formatter, "invalid date: '{field}' is {reason}")
    }
}

impl std::error::Error for InvalidDate {}

impl FromStr for Date {
    type Err = InvalidDate;

    /// Parses a date from a string, using a few well known formats. Currently looks for:
    ///
    /// - 'YYYY-MM-DD'
    ///
    /// (more will be added as needed)
    ///
    /// ```
    /// # use timestamp::{date, Date};
    /// let example_date = date!(2000 - 01 - 01);
    /// let date_string = example_date.to_string();
    ///
    /// // verify that the Display impl returns this string.
    /// assert_eq!(&date_string, "2000-01-01");
    ///
    /// let parsed = date_string.parse::<Date>().unwrap();
    /// assert_eq!(parsed, example_date);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! parse_component {
            ($iter:expr; $t:ty; $var:ident) => {{
                $iter
                    .next()
                    .ok_or_else(|| InvalidDate {
                        field: DateField::$var,
                        reason: InvalidDateReason::Missing,
                    })?
                    .parse::<$t>()
                    .map_err(|_| InvalidDate {
                        field: DateField::$var,
                        reason: InvalidDateReason::Invalid,
                    })?
            }};
        }

        let mut component_iter = s.trim().split('-');

        let year = parse_component!(component_iter; i16; Year);

        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(InvalidDate {
                field: DateField::Year,
                reason: InvalidDateReason::OutOfRange,
            });
        }

        let month = parse_component!(component_iter; u8; Month);

        let month = match Month::from_number(month) {
            Some(ok) => ok,
            None => {
                return Err(InvalidDate {
                    field: DateField::Month,
                    reason: InvalidDateReason::OutOfRange,
                });
            }
        };

        let day = parse_component!(component_iter; u8; Day);

        let max = month.days_in(year);

        if !(1..=max).contains(&day) {
            return Err(InvalidDate {
                field: DateField::Day,
                reason: InvalidDateReason::OutOfRange,
            });
        }

        Ok(Date {
            year,
            month,
            // SAFETY: we check that it's greater than one above
            day: unsafe { NonZeroU8::new_unchecked(day) },
        })
    }
}

impl Date {
    /// [`time::Date::MIN`], wrapped in [`Self`].
    pub const MIN: Self = Self {
        year: MIN_YEAR,
        month: Month::January,
        day: unsafe { NonZeroU8::new_unchecked(1) },
    };

    /// The first day with a non-negative year, '0000-01-01'.
    pub const ZERO: Self = Self {
        year: 0,
        month: Month::January,
        day: unsafe { NonZeroU8::new_unchecked(1) },
    };

    /// [`time::Date::MAX`], wrapped in [`Self`].
    pub const MAX: Self = Self {
        year: MAX_YEAR,
        month: Month::December,
        // SAFETY: days_in always returns a non-zero int.
        day: unsafe { NonZeroU8::new_unchecked(Month::December.days_in(MAX_YEAR)) },
    };

    /// Returns an empty builder instance to assemble a [`Date`].
    pub const fn builder() -> DateBuilder<(), (), ()> {
        DateBuilder::new()
    }

    /// Returns the year containing this [`Date`].
    #[inline]
    pub const fn year(&self) -> i16 {
        self.year
    }

    /// Returns the non-zero day in the mont this [`Date`] falls on.    
    #[inline]
    pub const fn day_non_zero(&self) -> NonZeroU8 {
        self.day
    }
    /// Returns the day in the mont this [`Date`] falls on.
    /// Identical to 'date.day_non_zero().get()'.   
    #[inline]
    pub const fn day(&self) -> u8 {
        self.day.get()
    }

    /// Returns the month containing this [`Date`].
    #[inline]
    pub const fn month(&self) -> Month {
        self.month
    }

    /// Computes the duration difference between 2 [`Date`]'s.
    #[inline]
    pub fn delta(self, rhs: Self) -> crate::Duration {
        crate::Duration::from_time(self.into_time() - rhs.into_time())
    }

    /// Const-able version of [`Ord::cmp`]. This is the internal method used by both
    /// [`Ord`]/[`PartialOrd`].
    pub const fn const_cmp(&self, rhs: &Self) -> Ordering {
        macro_rules! cmp_ret_if_neq {
            ($a:expr, $b:expr) => {{
                if $a > $b {
                    return std::cmp::Ordering::Greater;
                } else if $a < $b {
                    return std::cmp::Ordering::Less;
                }
            }};
        }

        cmp_ret_if_neq!(self.year, rhs.year);

        match self.month.const_cmp(&rhs.month) {
            Ordering::Equal => (),
            other => return other,
        }

        cmp_ret_if_neq!(self.day.get(), rhs.day.get());

        // if none of those return, we're equal.
        Ordering::Equal
    }

    /// Shortcut to calling 'date.delta(rhs).abs()'.
    #[inline]
    pub fn abs_delta(self, rhs: Self) -> crate::Duration {
        self.delta(rhs).abs()
    }

    /// Returns the components of this [`Date`], as a tuple of the year, month and day.
    pub const fn as_ymd(self) -> (i16, Month, u8) {
        (self.year, self.month, self.day.get())
    }

    /// Gets the next calender date, returning [`None`] if 'self' == [`Date::MAX`].
    #[inline]
    pub fn next_day(self) -> Option<Self> {
        if self == Self::MAX {
            return None;
        }

        let max_day = self.month.days_in(self.year);

        if self.day.get() < max_day {
            return Some(Self {
                year: self.year,
                month: self.month,
                // SAFETY: adding 1 to a number thats under 32 will always be non-zero, since it
                // cant overflow.
                day: unsafe { NonZeroU8::new_unchecked(self.day.get() + 1) },
            });
        }

        let month = self.month.next();

        let year = if month == Month::January {
            self.year + 1
        } else {
            self.year
        };

        Some(Self {
            year,
            month,
            day: unsafe { NonZeroU8::new_unchecked(1) },
        })
    }

    /// Writes 'self' into an existing [`fmt::Write`] type.
    ///
    /// See [`Date::write_into`] for the [`std::io::Write`] variant.
    pub fn format_into<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let mut buf = itoa::Buffer::new();

        macro_rules! ensure_2_digits {
            ($field:expr) => {{
                let s = buf.format($field);
                if s.len() == 1 {
                    w.write_str("0")?;
                }
                w.write_str(s)?;
            }};
        }

        if self.year < 0 {
            w.write_str("-")?;
        }

        let year = buf.format(self.year.abs());
        let prepended_zeros = 4_usize.saturating_sub(year.len());

        for _ in 0..prepended_zeros {
            w.write_str("0")?;
        }
        w.write_str(year)?;

        w.write_str("-")?;
        ensure_2_digits!(self.month as u8);
        w.write_str("-")?;
        ensure_2_digits!(self.day.get());
        Ok(())
    }

    /// Formats 'self' into an owned [`String`]. Infallible, unlike the [`fmt::Write`] methods.
    pub fn append_to_string(&self, dst: &mut String) {
        let mut buf = itoa::Buffer::new();

        macro_rules! ensure_2_digits {
            ($field:expr) => {{
                let s = buf.format($field);
                if s.len() == 1 {
                    dst.push('0');
                }
                dst.push_str(s);
            }};
        }

        if self.year < 0 {
            dst.push('-');
        }

        let year = buf.format(self.year.abs());
        let prepended_zeros = 4_usize.saturating_sub(year.len());

        for _ in 0..prepended_zeros {
            dst.push('0');
        }
        dst.push_str(year);

        dst.push('-');
        ensure_2_digits!(self.month as u8);
        dst.push('-');
        ensure_2_digits!(self.day.get());
    }

    /// Writes 'self' into a [`std::io::Write`] type.
    ///
    /// See [`Date::format_into`] for the [`std::fmt::Write`] variant.     
    pub fn write_into<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        write!(w, "{self}")
    }

    /// Gets the previous calender date, returning [`None`] if 'self' == [`Date::MIN`].
    #[inline]
    pub fn prev_day(self) -> Option<Self> {
        if self == Self::MIN {
            return None;
        }

        if self.day.get() != 1 {
            return Some(Self {
                year: self.year,
                month: self.month,
                // SAFETY: we checked that day is >1, so subtracting 1 will still be non-zero.
                day: unsafe { NonZeroU8::new_unchecked(self.day.get() - 1) },
            });
        }

        let month = self.month.previous();

        // if we rolled back into the previous year
        let year = if matches!(month, Month::December) {
            self.year - 1
        } else {
            self.year
        };

        // SAFETY: days_in should never return 0.
        let day = unsafe { NonZeroU8::new_unchecked(month.days_in(year)) };

        Some(Self { year, month, day })
    }

    /// Identical to [`Date::as_ymd`], but with the month converted to an integer (starting
    /// with January = 1).
    pub const fn as_ymd_int(self) -> (i16, u8, u8) {
        let (year, month, day) = self.as_ymd();

        (year, month as u8, day)
    }

    /// Converts from a [`chrono::NaiveDate`].    
    pub fn from_chrono_date_naive(date: chrono::NaiveDate) -> Self {
        Self {
            year: date.year() as i16,
            month: Month::from_number(date.month() as u8).expect("chrono gave a bad month"),
            day: NonZeroU8::new(date.day() as u8).expect("chrono gave a bad day (0)"),
        }
    }

    /// Returns the date in the local system time.
    pub fn today_local() -> Self {
        Self::from_chrono_date_naive(chrono::Local::now().date_naive())
    }

    /// Converts this [`Date`] into the earliest possible [`Timestamp`] on this date.
    /// (Midnight).
    ///
    /// This is a conveinence function that's identical to:
    /// ```
    /// # #[macro_use]
    /// # extern crate timestamp;
    /// # use timestamp::{Date, Time};
    /// # fn main() {
    /// let date: Date = timestamp::date!(2022 - 01 - 01);
    /// assert_eq!(date.earliest(), date.at_time(Time::MIN));
    /// # }
    /// ```
    pub const fn earliest(self) -> Timestamp {
        self.at_time(Time::MIN)
    }

    /// Adds a [`Duration`], returning [`None`] if the resulting [`Date`] is out of range.
    ///
    /// [`Duration`]: [`crate::Duration`]
    #[inline]
    pub fn checked_add(self, duration: crate::Duration) -> Option<Self> {
        self.into_time()
            .checked_add(duration.into_time_duration())
            .map(Self::from_time)
    }

    /// Adds a [`Duration`], saturating at the bounds if the resulting [`Date`] is out of range.
    ///
    /// [`Duration`]: [`crate::Duration`]
    #[inline]
    pub fn saturating_add(self, duration: crate::Duration) -> Self {
        self.into_time()
            .saturating_add(duration.into_time_duration())
            .into()
    }

    /// Subtracts a [`Duration`], saturating at the bounds if the resulting [`Date`] is out of
    /// range.
    ///
    /// [`Duration`]: [`crate::Duration`]
    #[inline]
    pub fn saturating_sub(self, duration: crate::Duration) -> Self {
        self.into_time()
            .saturating_sub(duration.into_time_duration())
            .into()
    }

    /// Subtracts a [`Duration`], returning [`None`] if the resulting [`Date`] is out of range.
    ///
    /// [`Duration`]: [`crate::Duration`]
    #[inline]
    pub fn checked_sub(self, duration: crate::Duration) -> Option<Self> {
        self.checked_add(-duration)
    }

    pub(crate) const fn from_time(date: time::Date) -> Self {
        Self {
            month: Month::from_time_month(date.month()),
            day: NonZeroU8::new(date.day())
                .expect("day should always be non-zero (and within 1..=31)"),
            year: date.year() as i16,
        }
    }

    pub(crate) const fn into_time(self) -> time::Date {
        match time::Date::from_calendar_date(
            self.year as i32,
            self.month.into_time_month(),
            self.day.get(),
        ) {
            Ok(date) => date,
            Err(_) => panic!("Date maintains the same set of invariants on value ranges"),
        }
    }

    /// Converts this [`Date`] into the latest possible [`Timestamp`] on this date.
    /// (23:59:59.999999999).
    ///
    /// This is a conveinence function that's identical to:
    /// ```
    /// # #[macro_use]
    /// # extern crate timestamp;
    /// # use timestamp::{Date, Time};
    /// # fn main() {
    /// let date: Date = timestamp::date!(2022 - 01 - 01);
    /// assert_eq!(date.latest(), date.at_time(Time::MAX));
    /// # }
    /// ```
    pub const fn latest(self) -> Timestamp {
        self.at_time(Time::MAX)
    }

    /// Returns the ordinal date.
    pub const fn to_ordinal(self) -> (i16, u16) {
        let (year, ord) = self.into_time().to_ordinal_date();
        (year as i16, ord)
    }

    /// Combines this [`Date`] with a [`Time`], to assemble a full [`Timestamp`].
    #[inline]
    pub const fn at_time(self, time: Time) -> Timestamp {
        Timestamp::from_offset_datetime(self.into_time().with_time(time.into_time()).assume_utc())
    }

    /// Returns the UTC [`Date`].
    pub fn today_utc() -> Self {
        Self::from_chrono_date_naive(chrono::Utc::now().date_naive())
    }

    #[allow(deprecated)]
    fn from_chrono_date<Tz>(date: chrono::Date<Tz>) -> Self
    where
        Tz: chrono::TimeZone,
    {
        let month = Month::from_number(date.month() as u8)
            .expect("docs for chrono::Date::month specify its within range");

        let year = date.year() as i16;

        let max_day = month.days_in(year);

        let day = date.day() as u8;

        if day > max_day || day == 0 {
            panic!("invalid day");
        }

        Self {
            // SAFETY: we panic above if 0.
            day: unsafe { NonZeroU8::new_unchecked(day) },
            year,
            month,
        }
    }
}

impl Step for Date {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        let (start_year, start_ord) = start.to_ordinal();
        let (end_year, end_ord) = end.to_ordinal();

        let delta_years = (start_year..=end_year)
            .map(|year| time::util::days_in_year(year as i32) as isize)
            .sum::<isize>();

        let delta_ord = end_ord as isize - start_ord as isize;

        let delta_days = delta_years + delta_ord;

        Some(delta_days.unsigned_abs())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        start.checked_add(count * crate::Duration::DAY)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        start.checked_sub(count * crate::Duration::DAY)
    }
}

#[test]
fn test_date_range() {
    let start = crate::date!(2022 - 1 - 3);
    let end = crate::date!(2022 - 1 - 5);

    let expecting = &[crate::date!(2022 - 1 - 3), crate::date!(2022 - 1 - 4)];

    let found = (start..end).collect::<Vec<_>>();
    assert_eq!(found.as_slice(), expecting);

    let count = (start..end).count();
    assert_eq!(count, 2);
}

#[allow(deprecated)]
impl<Tz> From<chrono::Date<Tz>> for Date
where
    Tz: chrono::TimeZone,
{
    fn from(date: chrono::Date<Tz>) -> Self {
        Self::from_chrono_date(date)
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        time::Date::deserialize(deserializer).map(Date::from_time)
    }
}

impl From<time::Date> for Date {
    fn from(d: time::Date) -> Self {
        Self::from_time(d)
    }
}

impl From<Date> for time::Date {
    fn from(d: Date) -> Self {
        d.into_time()
    }
}

impl ops::Sub for Date {
    type Output = crate::Duration;

    fn sub(self, other: Self) -> Self::Output {
        self.delta(other)
    }
}

impl ops::Sub for &Date {
    type Output = crate::Duration;

    fn sub(self, other: Self) -> Self::Output {
        self.delta(*other)
    }
}

impl ops::Sub<Date> for &Date {
    type Output = crate::Duration;

    fn sub(self, other: Date) -> Self::Output {
        self.delta(other)
    }
}

impl ops::Sub<&Date> for Date {
    type Output = crate::Duration;

    fn sub(self, other: &Date) -> Self::Output {
        self.delta(*other)
    }
}

impl ops::Add<crate::Duration> for Date {
    type Output = Self;

    fn add(self, dur: crate::Duration) -> Self {
        self.saturating_add(dur)
    }
}

impl ops::Add<crate::Duration> for &Date {
    type Output = Date;

    fn add(self, other: crate::Duration) -> Date {
        self.saturating_add(other)
    }
}

impl ops::Add<&crate::Duration> for &Date {
    type Output = Date;

    fn add(self, other: &crate::Duration) -> Date {
        self.saturating_add(*other)
    }
}

impl ops::Add<&crate::Duration> for Date {
    type Output = Date;

    fn add(self, other: &crate::Duration) -> Date {
        self.saturating_add(*other)
    }
}

impl ops::AddAssign<crate::Duration> for Date {
    fn add_assign(&mut self, rhs: crate::Duration) {
        *self = self.saturating_add(rhs);
    }
}

impl ops::Sub<crate::Duration> for Date {
    type Output = Self;

    fn sub(self, other: crate::Duration) -> Self {
        self.saturating_sub(other)
    }
}

impl ops::Sub<crate::Duration> for &Date {
    type Output = Date;

    fn sub(self, other: crate::Duration) -> Date {
        self.saturating_sub(other)
    }
}

impl ops::Sub<&crate::Duration> for &Date {
    type Output = Date;

    fn sub(self, other: &crate::Duration) -> Date {
        self.saturating_sub(*other)
    }
}

impl ops::Sub<&crate::Duration> for Date {
    type Output = Date;

    fn sub(self, other: &crate::Duration) -> Date {
        self.saturating_sub(*other)
    }
}

impl ops::SubAssign<crate::Duration> for Date {
    fn sub_assign(&mut self, rhs: crate::Duration) {
        *self = self.saturating_sub(rhs);
    }
}

#[test]
fn test_display() {
    let small_digits = date!(2022 - 1 - 1);
    assert_eq!(small_digits.to_string().as_str(), "2022-01-01");

    let mixed_digits = date!(2022 - 1 - 15);
    assert_eq!(mixed_digits.to_string().as_str(), "2022-01-15");

    let full_digits = date!(2022 - 10 - 15);
    assert_eq!(full_digits.to_string().as_str(), "2022-10-15");
}
