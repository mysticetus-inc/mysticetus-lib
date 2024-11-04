//! Parsing utilities for [`Timestamp`](crate::Timestamp)

use std::sync::atomic::AtomicBool;

use chrono_tz::Tz;
use time::PrimitiveDateTime;
use time::error::Parse;
use time::format_description::{Component, FormatItem, modifier};
use time::parsing::Parsed;

/// Only log about missing time zones once, otherwise we spam.
static LOGGED_MISSING_TZ: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Datetime(Parse),
    #[error(transparent)]
    UtcOffset(Parse),
    #[error("unknown timezone: '{0}'")]
    TimeZone(String),
}

impl<I> From<I> for ParseError
where
    I: Into<Parse>,
{
    fn from(err: I) -> Self {
        Self::Datetime(err.into())
    }
}

/// A format definition for primitive datetimes (i.e, no timezone or UTC offsets)
/// are parsed or taken into account.
///
/// This is hand-rolled for a couple reasons:
///
/// The docs for the syntax of [`format_description!`] is less than ideal, and has no mention of
/// how to specify either optional components (via [`FormatItem::Optional`]), or unions of
/// allowed items (via [`FormatItem::First`]).
///
/// In order to handle every format this version aims to handle, we'd need to define many different
/// whole format descriptions, and iterate over them to find the first match (inefficient).
///
/// The formats this aims to support, using rust ranges for values, | for unions between allowed
/// characters (where at least 1 is required), and `(...)?` for an optional group:
/// ```markdown
/// (whitespace between components for readabilty)
/// <code>
///   ----------------- date -----------------
///  -9999..=9999 '-'|'/' 1..=12 '-'|'/' 1..=31 'T'|' ' 0..=23 ':' 0..=59 ':' 0..=59 ('.' [0-9]+)?
///   -----------         ------         ------         ------     ------     ------       ----
///       year            month           day            hour       mins       secs      subsecond
/// </code>
/// ```
///
/// [`format_description!`]: time::macros::format_description!
const PRIMITIVE_DATETIME_FORMAT: &[FormatItem<'static>] = &[
    // year is always leading
    FormatItem::Component(Component::Year(modifier::Year::default())),
    // next component picks whether or not the date components are separated by
    // '-' or '/' ('-' is more common, and also the desired display format, so it's first).
    //
    // Once a separator is known, it parses the remainder of the date with that separator only
    // (to reject malformed date strings like '2022-01/12')
    FormatItem::First(&[
        FormatItem::Compound(&[
            FormatItem::Literal(b"-"),
            FormatItem::Component(Component::Month(modifier::Month::default())),
            FormatItem::Literal(b"-"),
            FormatItem::Component(Component::Day(modifier::Day::default())),
        ]),
        FormatItem::Compound(&[
            FormatItem::Literal(b"/"),
            FormatItem::Component(Component::Month(modifier::Month::default())),
            FormatItem::Literal(b"/"),
            FormatItem::Component(Component::Day(modifier::Day::default())),
        ]),
    ]),
    // Either a space or T separator between the date and time
    // (a space is what mysticetus does by default, so it's first)
    FormatItem::First(&[FormatItem::Literal(b" "), FormatItem::Literal(b"T")]),
    // time, in HH:MM:SS
    FormatItem::Component(Component::Hour(modifier::Hour::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Minute(modifier::Minute::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Second(modifier::Second::default())),
    // optional trailing subseconds, including the '.' separator
    FormatItem::Optional(&FormatItem::Compound(&[
        FormatItem::Literal(b"."),
        FormatItem::Component(Component::Subsecond(modifier::Subsecond::default())),
    ])),
];

#[inline(always)]
const fn unpadded_offset_hour() -> modifier::OffsetHour {
    let mut offset = modifier::OffsetHour::default();
    offset.sign_is_mandatory = false;
    offset.padding = modifier::Padding::None;
    offset
}

const SIMPLE_UTC_OFFSET_FORMAT: &[FormatItem<'static>] = &[
    FormatItem::Component(Component::OffsetHour(unpadded_offset_hour())),
    // the remaining offset minutes/seconds are optional, but we do require colons
    // if it exists. This is because we cant disambiguate where the hours end and
    // the minutes start in a string like this '+123...' (in theory seconds and
    // minutes will be zero padded, but thats an edge case id like to avoid, since
    // getting it wrong means incorrect offsets with no warnings/errors)
    FormatItem::Optional(&FormatItem::Compound(&[
        FormatItem::Literal(b":"),
        FormatItem::Component(Component::OffsetMinute(modifier::OffsetMinute::default())),
        FormatItem::Optional(&FormatItem::Compound(&[
            FormatItem::Literal(b":"),
            FormatItem::Component(Component::OffsetSecond(modifier::OffsetSecond::default())),
        ])),
    ])),
];

/// For formal UTC offsets, where the hours are 0 padded (i.e a bare single digit integer will fail)
const FORMAL_UTC_OFFSET_FORMAT: &[FormatItem<'static>] = &[
    FormatItem::Component(Component::OffsetHour(modifier::OffsetHour::default())),
    // the remaining offset minutes/seconds are optional, as are the colons.
    // The colons can be optional here, since we require zero padded numbers,
    // so we know all components will be 2 digits.
    FormatItem::Optional(&FormatItem::Compound(&[
        FormatItem::Optional(&FormatItem::Literal(b":")),
        FormatItem::Component(Component::OffsetMinute(modifier::OffsetMinute::default())),
        FormatItem::Optional(&FormatItem::Compound(&[
            FormatItem::Optional(&FormatItem::Literal(b":")),
            FormatItem::Component(Component::OffsetSecond(modifier::OffsetSecond::default())),
        ])),
    ])),
];

/// Attempts both [`FORMAL_UTC_OFFSET_FORMAT`] and [`SIMPLE_UTC_OFFSET_FORMAT`] (in that order),
/// returning the first one that succeeds.
const UTC_OFFSET_FORMAT: FormatItem<'static> = FormatItem::First(&[
    FormatItem::Compound(FORMAL_UTC_OFFSET_FORMAT),
    FormatItem::Compound(SIMPLE_UTC_OFFSET_FORMAT),
]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum OnMissingTz {
    #[default]
    Warn,
    Ignore,
}

/// Parse only the primitive datetime from the bytes of a datetime string. If successful, returns
/// the number of bytes consumed in parsing (that way we know where to start from when parsing the
/// timezone/UTC offset).
pub fn parse_primitive(dt: &str) -> Result<(usize, PrimitiveDateTime), ParseError> {
    let mut parser = Parsed::new();

    let mut dt_bytes = dt.as_bytes();

    for item in PRIMITIVE_DATETIME_FORMAT {
        dt_bytes = parser.parse_item(dt_bytes, item)?;
    }

    let prim_dt = parser.try_into()?;

    Ok((dt.len() - dt_bytes.len(), prim_dt))
}

fn parse_offset(offset: &str) -> Result<(usize, time::UtcOffset), ParseError> {
    let mut parser = Parsed::new();

    let remaining_bytes = parser.parse_item(offset.as_bytes(), &UTC_OFFSET_FORMAT)?;

    let parsed_offset: time::UtcOffset = parser
        .try_into()
        .map_err(|err| ParseError::UtcOffset(Parse::from(err)))?;

    Ok((offset.len() - remaining_bytes.len(), parsed_offset))
}

pub fn parse_timezone_or_offset(
    timestamp_str: &str,
    primitive: PrimitiveDateTime,
    tz_or_offset_bytes: &str,
    on_missing_tz: OnMissingTz,
) -> Result<crate::Timestamp, ParseError> {
    match tz_or_offset_bytes.trim() {
        // handle warning on an empty string/missing tz
        "" if on_missing_tz == OnMissingTz::Warn => {
            if !LOGGED_MISSING_TZ.swap(true, std::sync::atomic::Ordering::Relaxed) {
                tracing::warn!(message = "timestamp missing timezone", %timestamp_str);
            }

            Ok(crate::Timestamp::from(primitive.assume_utc()))
        }
        // Z, UTC, GMT or empty (if not warning) we treat as utc.
        // this lets us avoid more complex parsing below for simple cases that are known UTC.
        "" | "Z" | "UTC" | "GMT" => Ok(crate::Timestamp::from(primitive.assume_utc())),
        //
        mut offset_or_tz => {
            // if we have a leading UTC, we're likely in a format similar to 'UTC-5', where '-5' is
            // the offset. stripping the UTC prefix lets us parse that, or a raw numeric
            // offset in one go.
            if let Some(stripped) = offset_or_tz.strip_prefix("UTC") {
                offset_or_tz = stripped;
            }

            if is_possible_offset(offset_or_tz) {
                let (_, offset) = parse_offset(offset_or_tz)?;
                return Ok(crate::Timestamp::from(primitive.assume_offset(offset)));
            }

            let tz = complex_tz_lookup(offset_or_tz)?;
            Ok(combine_with_tz(primitive, tz))
        }
    }
}

fn complex_tz_lookup(offset_or_tz: &str) -> Result<chrono_tz::Tz, ParseError> {
    // This is a hard-coded map of timezones that Entiat/Misc/TimeStamp.cs
    // looks for, so until 'timestamp-tz' is up and running this will have to do.
    match offset_or_tz {
        "HST" => Ok(Tz::Pacific__Honolulu),
        "AKST" | "AKDT" => Ok(Tz::US__Alaska),
        "PST" | "PDT" => Ok(Tz::US__Pacific),
        "PSTmx" | "PDTmx" => Ok(Tz::America__Tijuana),
        "MST" | "MDT" => Ok(Tz::US__Mountain),
        "MSTmx" | "MDTmx" => Ok(Tz::America__Hermosillo),
        "CST" | "CDT" => Ok(Tz::US__Central),
        "CSTmx" | "CDTmx" => Ok(Tz::America__Mexico_City),
        "EST" | "EDT" => Ok(Tz::US__Eastern),
        "ESTmx" | "EDTmx" => Ok(Tz::America__Cancun),
        "PrST" => Ok(Tz::America__Asuncion),
        "AST" | "ADT" => Ok(Tz::Canada__Atlantic),
        "HKT" => Ok(Tz::Asia__Hong_Kong),
        "JST" => Ok(Tz::Asia__Tokyo),
        "ChST" => Ok(Tz::Pacific__Guam),
        "NZST" | "NZDT" => Ok(Tz::Pacific__Auckland),
        // try and parse as a last ditch effort
        _ => offset_or_tz.parse().map_err(ParseError::TimeZone),
    }
}

fn combine_with_tz(prim: PrimitiveDateTime, tz: Tz) -> crate::Timestamp {
    let (year, month, day) = prim.to_calendar_date();
    let (hour, min, sec, nano) = prim.as_hms_nano();

    let date = chrono::NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap();

    let time =
        chrono::NaiveTime::from_hms_nano_opt(hour as u32, min as u32, sec as u32, nano).unwrap();

    let tz_dt = date.and_time(time).and_local_timezone(tz).unwrap();

    crate::Timestamp::from_datetime(tz_dt)
}

fn is_possible_offset(s: &str) -> bool {
    s.chars()
        .all(|ch: char| matches!(ch, '0'..='9' | '+' | '-' | ':'))
}

pub fn parse_timestamp(
    s: &str,
    on_missing_tz: OnMissingTz,
) -> Result<crate::Timestamp, ParseError> {
    let (tz_start, primitive) = parse_primitive(s)?;

    let tz_str = s.get(tz_start..).unwrap_or("");

    parse_timezone_or_offset(s, primitive, tz_str, on_missing_tz)
}
