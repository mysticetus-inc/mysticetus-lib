use crate::protos::mysticetus::common::date::Month;
use crate::protos::mysticetus::common::Date;

impl const From<timestamp::Month> for Month {
    #[inline]
    fn from(month: timestamp::Month) -> Self {
        match month {
            timestamp::Month::January => Self::January,
            timestamp::Month::February => Self::February,
            timestamp::Month::March => Self::March,
            timestamp::Month::April => Self::April,
            timestamp::Month::May => Self::May,
            timestamp::Month::June => Self::June,
            timestamp::Month::July => Self::July,
            timestamp::Month::August => Self::August,
            timestamp::Month::September => Self::September,
            timestamp::Month::October => Self::October,
            timestamp::Month::November => Self::November,
            timestamp::Month::December => Self::December,
        }
    }
}

impl const From<Month> for timestamp::Month {
    #[inline]
    fn from(month: Month) -> Self {
        match month {
            Month::January => Self::January,
            Month::February => Self::February,
            Month::March => Self::March,
            Month::April => Self::April,
            Month::May => Self::May,
            Month::June => Self::June,
            Month::July => Self::July,
            Month::August => Self::August,
            Month::September => Self::September,
            Month::October => Self::October,
            Month::November => Self::November,
            Month::December => Self::December,
        }
    }
}

impl const From<timestamp::Date> for Date {
    fn from(date: timestamp::Date) -> Self {
        Date {
            month: date.month() as i32,
            day: date.day() as u32,
            year: date.year() as i32,
        }
    }
}

impl const From<Date> for timestamp::Date {
    #[inline]
    fn from(date: Date) -> Self {
        Self::builder()
            .year(date.year as i16)
            .month_digit(date.month as u8)
            .day(date.day as u8)
    }
}
