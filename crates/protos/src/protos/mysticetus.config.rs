/// Defines an IHA permit.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Permit {
    /// The name of the permit.
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The start date for the permit. Permits are only active for 1 year,
    /// though this may need changes once we incorperate renewed permits.
    #[prost(message, required, tag = "2")]
    pub start_date: super::common::Date,
    /// The number of takes allowed for each species.
    #[prost(map = "string, uint32", tag = "3")]
    pub species: ::std::collections::HashMap<::prost::alloc::string::String, u32>,
    /// The optional display ordering for the defined species.
    #[prost(string, repeated, tag = "4")]
    pub ordering: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The optional list of emails that are to be alerted.
    #[prost(string, repeated, tag = "5")]
    pub alert_emails: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Defines predefined into on a repository. Can be either ignored or
/// fully configured.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Repo {
    #[prost(oneof = "repo::Config", tags = "1, 2")]
    pub config: ::core::option::Option<repo::Config>,
}
/// Nested message and enum types in `Repo`.
pub mod repo {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Config {
        #[prost(enumeration = "super::True", tag = "1")]
        Ignored(i32),
        #[prost(message, tag = "2")]
        Repo(super::RepoConfig),
    }
}
/// A configured repo.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RepoConfig {
    /// The name of the client.
    #[prost(string, required, tag = "1")]
    pub client: ::prost::alloc::string::String,
    /// The lease areas that will be covered under this repo.
    #[prost(string, repeated, tag = "2")]
    pub lease_areas: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The name(s) of applicible permit(s). If no permits are specified,
    /// this repo is not bound by one.
    #[prost(string, repeated, tag = "3")]
    pub permit: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The name of the marine services provider.
    #[prost(string, required, tag = "4")]
    pub marine_services_provider: ::prost::alloc::string::String,
    /// The name of the PSO provider
    #[prost(string, required, tag = "5")]
    pub pso_provider: ::prost::alloc::string::String,
    /// The defined stations for this repo.
    #[prost(message, repeated, tag = "6")]
    pub stations: ::prost::alloc::vec::Vec<super::common::Station>,
    /// The survey type.
    #[prost(enumeration = "GeoType", required, tag = "7")]
    pub geo_type: i32,
    /// The optional year when this project is active
    #[prost(uint32, optional, tag = "8")]
    pub year: ::core::option::Option<u32>,
    /// The start date of the project. All data before this date should be treated
    /// as test/setup data.
    #[prost(message, required, tag = "9")]
    pub start_date: super::common::Date,
    /// An optional list of pm station ids
    #[prost(string, repeated, tag = "12")]
    pub pm_station_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional data mapping to get around non-standard template definitions.
    #[prost(message, optional, tag = "13")]
    pub data_mapping: ::core::option::Option<DataMapping>,
    /// An optional set of validation steps that should be skipped. i.e, a land
    /// based project would be exempt from GPS/GPX checks.
    #[prost(
        enumeration = "super::validation::ValidationKind",
        repeated,
        packed = "false",
        tag = "14"
    )]
    pub skip_validation: ::prost::alloc::vec::Vec<i32>,
    /// Whether or not this is a Mysticetus Crew project, and will have reports
    /// sent out automatically.
    #[prost(bool, optional, tag = "15")]
    pub automated_reports: ::core::option::Option<bool>,
    /// Then end date, or the \[`Active`\] sentinel flag.
    #[prost(oneof = "repo_config::ProjectEnd", tags = "10, 11")]
    pub project_end: ::core::option::Option<repo_config::ProjectEnd>,
}
/// Nested message and enum types in `RepoConfig`.
pub mod repo_config {
    /// Then end date, or the \[`Active`\] sentinel flag.
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ProjectEnd {
        /// A repo that's active with an unset end date.
        #[prost(enumeration = "super::Active", tag = "10")]
        Active(i32),
        /// The end date for the project. All data after this date should be treated
        /// as test data. Data from before the end date that's edited after the end
        /// date should still show up under it's original date.
        #[prost(message, tag = "11")]
        EndDate(super::super::common::Date),
    }
}
/// Data mapping for non-standard (i.e Vineyard) templates.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataMapping {
    /// Standard Entry sheet name.
    #[prost(map = "string, string", tag = "1")]
    pub data:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Mapped names for the files.
    #[prost(map = "string, string", tag = "2")]
    pub template:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Sentinel enum to represent a boolean that can only be true.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum True {
    True = 0,
}
impl True {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            True::True => "TRUE",
        }
    }
}
/// Sentinel enum to represent the 'Active' string used for an ongoing project
/// with no set end date.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Active {
    Active = 0,
}
impl Active {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Active::Active => "ACTIVE",
        }
    }
}
/// The type of survey a project is performing.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GeoType {
    /// An unspecified / unknown geo type.
    Unspecified = 0,
    /// Geotechnical surveys
    Gt = 1,
    /// Geophysical surveys
    Gp = 2,
    /// Bethnic surveys
    Be = 3,
}
impl GeoType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GeoType::Unspecified => "UNSPECIFIED",
            GeoType::Gt => "GT",
            GeoType::Gp => "GP",
            GeoType::Be => "BE",
        }
    }
}
