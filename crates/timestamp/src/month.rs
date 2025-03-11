//! [`Month`] defintion + impls.
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::Step;

/// A month in the year.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(missing_docs)] // Dont think we __need__ to document each month.
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl PartialEq for Month {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidMonth(pub u8);

impl fmt::Display for InvalidMonth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is not a valid month, expected a value 1..=12",
            self.0
        )
    }
}

impl std::error::Error for InvalidMonth {}

impl TryFrom<u8> for Month {
    type Error = InvalidMonth;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match Self::from_number(value) {
            Some(month) => Ok(month),
            None => Err(InvalidMonth(value)),
        }
    }
}

impl Eq for Month {}

impl PartialOrd for Month {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Month {
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(other)
    }
}

impl Hash for Month {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_u8(*self as u8)
    }
}

impl Month {
    /// All 12 months, in order.
    pub const ALL: [Self; 12] = [
        Self::January,
        Self::February,
        Self::March,
        Self::April,
        Self::May,
        Self::June,
        Self::July,
        Self::August,
        Self::September,
        Self::October,
        Self::November,
        Self::December,
    ];

    /// Builds a [`Month`] from the corresponding digit, starting at '[`Month::January`] = 1'
    #[inline]
    pub const fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::January),
            2 => Some(Self::February),
            3 => Some(Self::March),
            4 => Some(Self::April),
            5 => Some(Self::May),
            6 => Some(Self::June),
            7 => Some(Self::July),
            8 => Some(Self::August),
            9 => Some(Self::September),
            10 => Some(Self::October),
            11 => Some(Self::November),
            12 => Some(Self::December),
            _ => None,
        }
    }

    /// Const-able [`Ord::cmp`]. Used internally by the [`Ord`]/[`PartialOrd`] impls.
    #[inline]
    pub const fn const_cmp(&self, other: &Self) -> Ordering {
        let a = *self as u8;
        let b = *other as u8;

        if a > b {
            Ordering::Greater
        } else if a < b {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }

    /// Returns the previous month, wrapping to [`December`] if 'self == [`January`]'
    ///
    /// [`December`]: [`Month::December`]
    /// [`January`]: [`Month::January`]
    #[inline]
    pub const fn previous(self) -> Self {
        match self {
            Self::January => Self::December,
            Self::February => Self::January,
            Self::March => Self::February,
            Self::April => Self::March,
            Self::May => Self::April,
            Self::June => Self::May,
            Self::July => Self::June,
            Self::August => Self::July,
            Self::September => Self::August,
            Self::October => Self::September,
            Self::November => Self::October,
            Self::December => Self::November,
        }
    }

    pub(crate) const fn into_time_month(self) -> time::Month {
        match self {
            Self::January => time::Month::January,
            Self::February => time::Month::February,
            Self::March => time::Month::March,
            Self::April => time::Month::April,
            Self::May => time::Month::May,
            Self::June => time::Month::June,
            Self::July => time::Month::July,
            Self::August => time::Month::August,
            Self::September => time::Month::September,
            Self::October => time::Month::October,
            Self::November => time::Month::November,
            Self::December => time::Month::December,
        }
    }

    pub(crate) const fn from_time_month(month: time::Month) -> Self {
        match month {
            time::Month::January => Self::January,
            time::Month::February => Self::February,
            time::Month::March => Self::March,
            time::Month::April => Self::April,
            time::Month::May => Self::May,
            time::Month::June => Self::June,
            time::Month::July => Self::July,
            time::Month::August => Self::August,
            time::Month::September => Self::September,
            time::Month::October => Self::October,
            time::Month::November => Self::November,
            time::Month::December => Self::December,
        }
    }

    /// Returns the number of days in this [`Month`], given the year.
    pub const fn days_in(&self, year: i16) -> u8 {
        self.into_time_month().length(year as i32)
    }

    /// Returns the next month, wrapping to [`January`] if 'self == [`December`]'
    ///
    /// [`December`]: [`Month::December`]
    /// [`January`]: [`Month::January`]
    #[inline]
    pub const fn next(self) -> Self {
        match self {
            Self::January => Self::February,
            Self::February => Self::March,
            Self::March => Self::April,
            Self::April => Self::May,
            Self::May => Self::June,
            Self::June => Self::July,
            Self::July => Self::August,
            Self::August => Self::September,
            Self::September => Self::October,
            Self::October => Self::November,
            Self::November => Self::December,
            Self::December => Self::January,
        }
    }
}

impl Step for Month {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        let diff = (*start as u8).abs_diff(*end as u8) as usize;
        (diff, Some(diff))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        match count {
            0 => Some(start),
            1 => Some(start.next()),
            _ => {
                // normalize to 0..=11.
                let m = start as u8 as usize - 1;
                // get the modulo of the count so we have an offset that wont overflow
                let normalized_count = count % 12;
                // add + take the modulo again to wrap back into the 0-11 valid range
                let new_m = (m + normalized_count) % 12;

                Month::from_number((new_m + 1) as u8)
            }
        }
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        match count {
            0 => Some(start),
            1 => Some(start.previous()),
            _ => {
                // normalize to 0..=11.
                let m = start as u8 as usize - 1;
                // offset by 1 'year' so we're within 12..=23, that way we can subtract the
                // normalized offset without overflowing.
                let offset_m = m + 12;
                // get the modulo of the count so we have an offset that wont overflow
                let normalized_offset = count % 12;

                // subtract + take the modulo again to wrap back into the 0-11 valid range
                let new_m = (offset_m - normalized_offset) % 12;

                Month::from_number((new_m + 1) as u8)
            }
        }
    }
}

impl From<Month> for time::Month {
    fn from(month: Month) -> Self {
        // Both this and time::Month are repr(u8), with the same values for the corresponding
        // variants.
        unsafe { std::mem::transmute(month) }
    }
}

impl From<time::Month> for Month {
    fn from(month: time::Month) -> Self {
        // Both this and time::Month are repr(u8), with the same values for the corresponding
        // variants.
        unsafe { std::mem::transmute(month) }
    }
}

#[cfg(test)]
mod tests {
    use super::{Month, Step};

    #[test]
    fn test_month_transmute() {
        let mut time_month = time::Month::January;
        for month in Month::January..=Month::December {
            assert_eq!(month, time_month.into());
            time_month = time_month.next();
        }
    }

    #[test]
    fn test_month_step() {
        assert_eq!(Month::December.next(), Month::January);

        // basic test within the same year
        assert_eq!(
            Step::forward_checked(Month::January, 10),
            Some(Month::November)
        );

        // test wrapping exactly 1 year
        assert_eq!(Step::forward_checked(Month::March, 12), Some(Month::March));

        // test wrapping into the next year with an offset.
        assert_eq!(
            Step::forward_checked(Month::January, 13),
            Some(Month::February)
        );

        // repeat the same tests, but backwards
        // basic test within the same year
        assert_eq!(
            Step::backward_checked(Month::November, 10),
            Some(Month::January)
        );

        // test wrapping exactly 1 year
        assert_eq!(Step::backward_checked(Month::March, 12), Some(Month::March));

        // test wrapping into the next year with an offset.
        assert_eq!(
            Step::backward_checked(Month::February, 13),
            Some(Month::January)
        );
    }
}
