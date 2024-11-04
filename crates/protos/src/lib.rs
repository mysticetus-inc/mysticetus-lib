#![feature(const_trait_impl)]
// prevents clippy warnings in all generated code
#![allow(clippy::all)]

//! Contains prost generated types for Google proto/gRPC definitions.
//!
//! The main purpose of splitting this off into its own crate, is to prevent the invalid "doctests"
//! that generated from the proto definition comments.
//!
//! In order to reduce compile times, nearly all functionality is behind features, aside from 3
//! univerally required protos ([`google.api`], [`google.protobuf`] and [`google.rpc`]).
//!
//! Current feature flags are:
//! - `bigquery`: Enables the BigQuery V2 + Storage API, and associated type definitions
//! - `firestore`: Enables the client side Firestore API.
//! - `firestore-admin`: Infers `firestore`, and also enables the admin side of the API.
//! - `monitoring`: Enables the GCP Monitoring API.
//!
//! [`google.api`]: `protos::google::api`
//! [`google.protobuf`]: `protos::google::protobuf`
//! [`google.rpc`]: `protos::google::rpc`

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

mod impls;
mod protos;

#[cfg(feature = "api")]
pub use protos::google::api;
#[cfg(feature = "bigquery")]
pub use protos::google::cloud::bigquery::storage::v1 as bigquery_storage;
#[cfg(feature = "bigquery")]
pub use protos::google::cloud::bigquery::v2 as bigquery_v2;
#[cfg(feature = "cloud-run")]
pub use protos::google::cloud::run::v2 as cloud_run;
#[cfg(feature = "artifact-registry")]
pub use protos::google::devtools::artifactregistry::v1 as artifact_registry;
#[cfg(feature = "firestore-admin")]
pub use protos::google::firestore::admin::v1 as firestore_admin;
#[cfg(feature = "firestore")]
pub use protos::google::firestore::v1 as firestore;
#[cfg(feature = "iam")]
pub use protos::google::iam::v1 as iam;
#[cfg(feature = "logging")]
pub use protos::google::logging;
#[cfg(feature = "longrunning")]
pub use protos::google::longrunning;
#[cfg(feature = "monitoring")]
pub use protos::google::monitoring::v3 as monitoring;
#[cfg(feature = "protobuf")]
pub use protos::google::protobuf;
#[cfg(feature = "pubsub")]
pub use protos::google::pubsub::v1 as pubsub;
#[cfg(feature = "rpc")]
pub use protos::google::rpc;
#[cfg(feature = "storage")]
pub use protos::google::storage::v2 as storage;
#[cfg(feature = "type")]
pub use protos::google::r#type;

/*
#[cfg(feature = "mysticetus")]
pub use protos::mysticetus;
*/

#[cfg(any(
    feature = "spanner",
    feature = "spanner-admin-database",
    feature = "spanner-admin-instance",
))]
pub mod spanner {
    #[cfg(feature = "spanner")]
    pub use crate::protos::google::spanner::v1::*;

    #[cfg(any(feature = "spanner-admin-database", feature = "spanner-admin-instance"))]
    pub mod admin {
        #[cfg(feature = "spanner-admin-database")]
        pub use crate::protos::google::spanner::admin::database::v1 as database;
        #[cfg(feature = "spanner-admin-instance")]
        pub use crate::protos::google::spanner::admin::instance::v1 as instance;
    }
}

/// Enum for the common GCP Regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    /// us-central1
    UsCentral1,
    /// us-east1
    UsEast1,
    /// us-east4
    UsEast4,
    /// us-west1
    UsWest1,
    /// us-west2
    UsWest2,
    /// us-west3
    UsWest3,
    /// us-west4
    UsWest4,
    /// southamerica-east1
    SouthAmericaEast1,
    /// northamerica-northeast1
    NorthAmericaNortheast1,
    /// europe-central2
    EuropeCentral2,
    /// europe-north1
    EuropeNorth1,
    /// europe-west1
    EuropeWest1,
    /// europe-west2
    EuropeWest2,
    /// europe-west3
    EuropeWest3,
    /// europe-west4
    EuropeWest4,
    /// europe-west6
    EuropeWest6,
    /// australia-southeast1
    AustraliaSoutheast1,
    /// australia-southeast2
    AustraliaSoutheast2,
    /// asia-east1
    AsiaEast1,
    /// asia-east2
    AsiaEast2,
    /// asia-northeast1
    AsiaNortheast1,
    /// asia-northeast2
    AsiaNortheast2,
    /// asia-northeast3
    AsiaNortheast3,
    /// asia-south1
    AsiaSouth1,
    /// asia-south2
    AsiaSouth2,
    /// asia-southeast1
    AsiaSoutheast1,
    /// asia-southeast2
    AsiaSoutheast2,
}

impl Default for Region {
    /// Returns the most common region we use GCP resources, [`Region::UsCentral1`] ('us-centra1')
    fn default() -> Self {
        Self::UsCentral1
    }
}

impl Region {
    /// Returns the region name as [`&'static str`].
    ///
    /// ```
    /// assert_eq!(Region::UsCentral1.as_str(), "us-central1");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UsCentral1 => "us-central1",
            Self::UsEast1 => "us-east1",
            Self::UsEast4 => "us-east4",
            Self::UsWest1 => "us-west1",
            Self::UsWest2 => "us-west2",
            Self::UsWest3 => "us-west3",
            Self::UsWest4 => "us-west4",
            Self::SouthAmericaEast1 => "southamerica-east1",
            Self::NorthAmericaNortheast1 => "northamerica-northeast1",
            Self::EuropeCentral2 => "europe-central2",
            Self::EuropeNorth1 => "europe-north1",
            Self::EuropeWest1 => "europe-west1",
            Self::EuropeWest2 => "europe-west2",
            Self::EuropeWest3 => "europe-west3",
            Self::EuropeWest4 => "europe-west4",
            Self::EuropeWest6 => "europe-west6",
            Self::AustraliaSoutheast1 => "australia-southeast1",
            Self::AustraliaSoutheast2 => "australia-southeast2",
            Self::AsiaEast1 => "asia-east1",
            Self::AsiaEast2 => "asia-east2",
            Self::AsiaNortheast1 => "asia-northeast1",
            Self::AsiaNortheast2 => "asia-northeast2",
            Self::AsiaNortheast3 => "asia-northeast3",
            Self::AsiaSouth1 => "asia-south1",
            Self::AsiaSouth2 => "asia-south2",
            Self::AsiaSoutheast1 => "asia-southeast1",
            Self::AsiaSoutheast2 => "asia-southeast2",
        }
    }

    /// Returns whether or not this region is in the US.
    pub fn is_us(&self) -> bool {
        matches!(
            self,
            Self::UsCentral1
                | Self::UsEast1
                | Self::UsEast4
                | Self::UsWest1
                | Self::UsWest2
                | Self::UsWest3
                | Self::UsWest4
        )
    }
}

impl AsRef<str> for Region {
    /// Defers to [`Region::as_str`]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Region {
    /// Writes the string returned from [`Region::as_str`]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Error type from the [`FromStr`] impl on [`Region`]. Contains an owned copy of the invalid
/// string recieved by [`FromStr::from_str`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InvalidRegion {
    string: String,
}

impl InvalidRegion {
    /// Returns a reference to the invalid region string.
    pub fn invalid_region(&self) -> &str {
        self.string.as_str()
    }

    /// Consumes this error, and returns the owned string with the invalid region.
    pub fn into_invalid_region(self) -> String {
        self.string
    }
}

impl fmt::Display for InvalidRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "'{}' is an invalid GCP region identifier",
            self.string
        )
    }
}

impl std::error::Error for InvalidRegion {}

impl FromStr for Region {
    type Err = InvalidRegion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            r if r.eq_ignore_ascii_case("us-central1") => Ok(Self::UsCentral1),
            r if r.eq_ignore_ascii_case("us-east1") => Ok(Self::UsEast1),
            r if r.eq_ignore_ascii_case("us-east4") => Ok(Self::UsEast4),
            r if r.eq_ignore_ascii_case("us-west1") => Ok(Self::UsWest1),
            r if r.eq_ignore_ascii_case("us-west2") => Ok(Self::UsWest2),
            r if r.eq_ignore_ascii_case("us-west3") => Ok(Self::UsWest3),
            r if r.eq_ignore_ascii_case("us-west4") => Ok(Self::UsWest4),
            r if r.eq_ignore_ascii_case("southamerica-east1") => Ok(Self::SouthAmericaEast1),
            r if r.eq_ignore_ascii_case("northamerica-northeast1") => {
                Ok(Self::NorthAmericaNortheast1)
            }
            r if r.eq_ignore_ascii_case("europe-central2") => Ok(Self::EuropeCentral2),
            r if r.eq_ignore_ascii_case("europe-north1") => Ok(Self::EuropeNorth1),
            r if r.eq_ignore_ascii_case("europe-west1") => Ok(Self::EuropeWest1),
            r if r.eq_ignore_ascii_case("europe-west2") => Ok(Self::EuropeWest2),
            r if r.eq_ignore_ascii_case("europe-west3") => Ok(Self::EuropeWest3),
            r if r.eq_ignore_ascii_case("europe-west4") => Ok(Self::EuropeWest4),
            r if r.eq_ignore_ascii_case("europe-west6") => Ok(Self::EuropeWest6),
            r if r.eq_ignore_ascii_case("australia-southeast1") => Ok(Self::AustraliaSoutheast1),
            r if r.eq_ignore_ascii_case("australia-southeast2") => Ok(Self::AustraliaSoutheast2),
            r if r.eq_ignore_ascii_case("asia-east1") => Ok(Self::AsiaEast1),
            r if r.eq_ignore_ascii_case("asia-east2") => Ok(Self::AsiaEast2),
            r if r.eq_ignore_ascii_case("asia-northeast1") => Ok(Self::AsiaNortheast1),
            r if r.eq_ignore_ascii_case("asia-northeast2") => Ok(Self::AsiaNortheast2),
            r if r.eq_ignore_ascii_case("asia-northeast3") => Ok(Self::AsiaNortheast3),
            r if r.eq_ignore_ascii_case("asia-south1") => Ok(Self::AsiaSouth1),
            r if r.eq_ignore_ascii_case("asia-south2") => Ok(Self::AsiaSouth2),
            r if r.eq_ignore_ascii_case("asia-southeast1") => Ok(Self::AsiaSoutheast1),
            r if r.eq_ignore_ascii_case("asia-southeast2") => Ok(Self::AsiaSoutheast2),
            r => {
                return Err(InvalidRegion {
                    string: r.to_owned(),
                });
            }
        }
    }
}

impl Serialize for Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Region {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RegionVisitor)
    }
}

struct RegionVisitor;

impl<'de> de::Visitor<'de> for RegionVisitor {
    type Value = Region;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid GCP region identifier (i.e 'us-central1')")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let string = std::str::from_utf8(v).map_err(de::Error::custom)?;

        self.visit_str(string)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<Region>() {
            Ok(region) => Ok(region),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
        }
    }
}
