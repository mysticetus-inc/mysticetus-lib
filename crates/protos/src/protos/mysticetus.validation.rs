/// The result of running 1 validation check.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidationResult {
    /// The kind of check that was run.
    #[prost(enumeration = "ValidationKind", required, tag = "1")]
    pub kind: i32,
    /// The status of the validation check
    #[prost(enumeration = "ValidationStatus", required, tag = "2")]
    pub status: i32,
    /// When this validation check was run
    #[prost(message, required, tag = "3")]
    pub validated_at: super::super::google::protobuf::Timestamp,
    /// An optional filename to describe the source data of the check. Not
    /// specifically tied to '.Mysticetus' files, i.e for the `GPX_UPLOAD` check,
    /// this should be the name (and maybe path) of the .gpx file used.
    #[prost(string, optional, tag = "4")]
    pub file: ::core::option::Option<::prost::alloc::string::String>,
    /// Notes + debug info from validation.
    #[prost(string, repeated, tag = "5")]
    pub notes: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DailyValidation {
    /// The mysticetus repo that this validation belongs to.
    #[prost(string, required, tag = "1")]
    pub repo: ::prost::alloc::string::String,
    /// The Mysticetus file used in validation.
    #[prost(string, required, tag = "2")]
    pub mysticetus_file: ::prost::alloc::string::String,
    /// The type of the mysticetus file above.
    #[prost(enumeration = "super::common::FileType", required, tag = "3")]
    pub file_type: i32,
    /// The Station ID the mysticetus file was saved under.
    #[prost(message, required, tag = "4")]
    pub station_id: super::common::Station,
    /// The date this validation applies to.
    #[prost(message, required, tag = "5")]
    pub date: super::common::Date,
    /// The fully qualified repo-based path that points to the corresponding
    /// Mysticetus file. Must be a combination of the fields above, in the
    /// path order:
    ///
    /// `{repo}/{file_type}/{station_id}/{date}/{mysticetus_file}`
    #[prost(string, required, tag = "6")]
    pub qualified_file_path: ::prost::alloc::string::String,
    /// The results from different checks. The string key is expected to be the
    /// string representation of the `ValidationKind` variant specified in the
    /// `ValidationResult.kind` value.
    #[prost(map = "string, message", tag = "7")]
    pub checks: ::std::collections::HashMap<::prost::alloc::string::String, ValidationResult>,
}
/// The different types of validation checks we run. Meant to be extended with
/// future checks.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ValidationKind {
    /// An unspecified check. Should never be used, but catching this will
    /// be easier and more transparent than having a real variant used as a
    /// default value.
    Unspecified = 0,
    /// Checks that all rows are valid.
    DataEntry = 1,
    /// Checks that an analysis file exists for a certain day.
    EndOfDayQa = 2,
    /// Check that a daily report was generated for a given day
    LeadPsoDailyReport = 3,
    /// Checks that exposure counts were added against a certaint IHA.
    PmExposureConf = 4,
    /// Checks that there's a signed off file for a given day.
    PmSignOff = 5,
    /// Verify the GPS track is valid accross every effort row.
    GpsCoverage = 6,
    /// Verify there's a GPX file that covers a given day
    GpxUpload = 7,
}
impl ValidationKind {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ValidationKind::Unspecified => "UNSPECIFIED",
            ValidationKind::DataEntry => "DATA_ENTRY",
            ValidationKind::EndOfDayQa => "END_OF_DAY_QA",
            ValidationKind::LeadPsoDailyReport => "LEAD_PSO_DAILY_REPORT",
            ValidationKind::PmExposureConf => "PM_EXPOSURE_CONF",
            ValidationKind::PmSignOff => "PM_SIGN_OFF",
            ValidationKind::GpsCoverage => "GPS_COVERAGE",
            ValidationKind::GpxUpload => "GPX_UPLOAD",
        }
    }
}
/// The status for a specific validation check.
///
/// The UI representaion of these are as follows:
///   - COMPLETE, DOCKED => green gumball
///   - NOT_REQUIRED => white + black gumball
///   - PENDING => yellow warning gumball
///   - MISSING, ERROR => red error gumball
///   - UNKNOWN => greyed out gumball (this may not be the ideal depending on what causes the
///     status)
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ValidationStatus {
    /// A check that can't be performed for some reason. If used, notes should be
    /// added to describe why. This is also the default value.
    Unknown = 0,
    /// A check who's requirements have been fully met.
    Complete = 1,
    /// A check that's considered identical to \[`ValidationStatus::Complete`\]
    /// since the vessel was docked.
    Docked = 2,
    /// A check that's not required. I.e IHA Permit related checks on a project
    /// that isn't bound by a permit.
    NotRequired = 3,
    /// A check that failed, but is within a grace period (which can differs
    /// between checks).
    Pending = 4,
    /// A check who's requirements weren't met. See \[`ValidationStatus::Error`\]
    /// for programming/internal errors.
    Missing = 5,
    /// An internal error occured when running checks.
    Error = 6,
}
impl ValidationStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ValidationStatus::Unknown => "UNKNOWN",
            ValidationStatus::Complete => "COMPLETE",
            ValidationStatus::Docked => "DOCKED",
            ValidationStatus::NotRequired => "NOT_REQUIRED",
            ValidationStatus::Pending => "PENDING",
            ValidationStatus::Missing => "MISSING",
            ValidationStatus::Error => "ERROR",
        }
    }
}
