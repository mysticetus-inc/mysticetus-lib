// This file is @generated by prost-build.
/// An object that represents a latitude/longitude pair. This is expressed as a
/// pair of doubles to represent degrees latitude and degrees longitude. Unless
/// specified otherwise, this must conform to the
/// <a href="<http://www.unoosa.org/pdf/icg/2012/template/WGS_84.pdf">WGS84>
/// standard</a>. Values must be within normalized ranges.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(PartialOrd)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct LatLng {
    /// The latitude in degrees. It must be in the range \[-90.0, +90.0\].
    #[prost(double, tag = "1")]
    pub latitude: f64,
    /// The longitude in degrees. It must be in the range \[-180.0, +180.0\].
    #[prost(double, tag = "2")]
    pub longitude: f64,
}
/// Represents a day of the week.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DayOfWeek {
    /// The day of the week is unspecified.
    Unspecified = 0,
    /// Monday
    Monday = 1,
    /// Tuesday
    Tuesday = 2,
    /// Wednesday
    Wednesday = 3,
    /// Thursday
    Thursday = 4,
    /// Friday
    Friday = 5,
    /// Saturday
    Saturday = 6,
    /// Sunday
    Sunday = 7,
}
impl DayOfWeek {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "DAY_OF_WEEK_UNSPECIFIED",
            Self::Monday => "MONDAY",
            Self::Tuesday => "TUESDAY",
            Self::Wednesday => "WEDNESDAY",
            Self::Thursday => "THURSDAY",
            Self::Friday => "FRIDAY",
            Self::Saturday => "SATURDAY",
            Self::Sunday => "SUNDAY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DAY_OF_WEEK_UNSPECIFIED" => Some(Self::Unspecified),
            "MONDAY" => Some(Self::Monday),
            "TUESDAY" => Some(Self::Tuesday),
            "WEDNESDAY" => Some(Self::Wednesday),
            "THURSDAY" => Some(Self::Thursday),
            "FRIDAY" => Some(Self::Friday),
            "SATURDAY" => Some(Self::Saturday),
            "SUNDAY" => Some(Self::Sunday),
            _ => None,
        }
    }
}
/// A `CalendarPeriod` represents the abstract concept of a time period that has
/// a canonical start. Grammatically, "the start of the current
/// `CalendarPeriod`." All calendar times begin at midnight UTC.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CalendarPeriod {
    /// Undefined period, raises an error.
    Unspecified = 0,
    /// A day.
    Day = 1,
    /// A week. Weeks begin on Monday, following
    /// [ISO 8601](<https://en.wikipedia.org/wiki/ISO_week_date>).
    Week = 2,
    /// A fortnight. The first calendar fortnight of the year begins at the start
    /// of week 1 according to
    /// [ISO 8601](<https://en.wikipedia.org/wiki/ISO_week_date>).
    Fortnight = 3,
    /// A month.
    Month = 4,
    /// A quarter. Quarters start on dates 1-Jan, 1-Apr, 1-Jul, and 1-Oct of each
    /// year.
    Quarter = 5,
    /// A half-year. Half-years start on dates 1-Jan and 1-Jul.
    Half = 6,
    /// A year.
    Year = 7,
}
impl CalendarPeriod {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "CALENDAR_PERIOD_UNSPECIFIED",
            Self::Day => "DAY",
            Self::Week => "WEEK",
            Self::Fortnight => "FORTNIGHT",
            Self::Month => "MONTH",
            Self::Quarter => "QUARTER",
            Self::Half => "HALF",
            Self::Year => "YEAR",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CALENDAR_PERIOD_UNSPECIFIED" => Some(Self::Unspecified),
            "DAY" => Some(Self::Day),
            "WEEK" => Some(Self::Week),
            "FORTNIGHT" => Some(Self::Fortnight),
            "MONTH" => Some(Self::Month),
            "QUARTER" => Some(Self::Quarter),
            "HALF" => Some(Self::Half),
            "YEAR" => Some(Self::Year),
            _ => None,
        }
    }
}
/// Represents a textual expression in the Common Expression Language (CEL)
/// syntax. CEL is a C-like expression language. The syntax and semantics of CEL
/// are documented at <https://github.com/google/cel-spec.>
///
/// Example (Comparison):
///
///      title: "Summary size limit"
///      description: "Determines if a summary is less than 100 chars"
///      expression: "document.summary.size() < 100"
///
/// Example (Equality):
///
///      title: "Requestor is owner"
///      description: "Determines if requestor is the document owner"
///      expression: "document.owner == request.auth.claims.email"
///
/// Example (Logic):
///
///      title: "Public documents"
///      description: "Determine whether the document should be publicly visible"
///      expression: "document.type != 'private' && document.type != 'internal'"
///
/// Example (Data Manipulation):
///
///      title: "Notification string"
///      description: "Create a notification string with a timestamp."
///      expression: "'New message received at ' + string(document.create_time)"
///
/// The exact variables and functions that may be referenced within an expression
/// are determined by the service that evaluates it. See the service
/// documentation for additional information.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Expr {
    /// Textual representation of an expression in Common Expression Language
    /// syntax.
    #[prost(string, tag = "1")]
    pub expression: ::prost::alloc::string::String,
    /// Optional. Title for the expression, i.e. a short string describing
    /// its purpose. This can be used e.g. in UIs which allow to enter the
    /// expression.
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    /// Optional. Description of the expression. This is a longer text which
    /// describes the expression, e.g. when hovered over it in a UI.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Optional. String indicating the location of the expression for error
    /// reporting, e.g. a file name and a position in the file.
    #[prost(string, tag = "4")]
    pub location: ::prost::alloc::string::String,
}
/// Represents a whole or partial calendar date, such as a birthday. The time of
/// day and time zone are either specified elsewhere or are insignificant. The
/// date is relative to the Gregorian Calendar. This can represent one of the
/// following:
///
/// * A full date, with non-zero year, month, and day values
/// * A month and day value, with a zero year, such as an anniversary
/// * A year on its own, with zero month and day values
/// * A year and month value, with a zero day, such as a credit card expiration
/// date
///
/// Related types are [google.type.TimeOfDay][google.type.TimeOfDay] and
/// `google.protobuf.Timestamp`.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Date {
    /// Year of the date. Must be from 1 to 9999, or 0 to specify a date without
    /// a year.
    #[prost(int32, tag = "1")]
    pub year: i32,
    /// Month of a year. Must be from 1 to 12, or 0 to specify a year without a
    /// month and day.
    #[prost(int32, tag = "2")]
    pub month: i32,
    /// Day of a month. Must be from 1 to 31 and valid for the year and month, or 0
    /// to specify a year by itself or a year and month where the day isn't
    /// significant.
    #[prost(int32, tag = "3")]
    pub day: i32,
}
