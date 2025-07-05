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
#[cfg(feature = "tasks")]
pub use protos::google::cloud::tasks::v2 as tasks;
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
