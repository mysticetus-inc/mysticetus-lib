const ROOT: &[&str] = &["../../googleapis"];

const FILES: &[&str] = &[
    // BigQuery
    "../../googleapis/google/cloud/bigquery/v2/model.proto",
    "../../googleapis/google/cloud/bigquery/storage/v1/storage.proto",
    // Firestore
    "../../googleapis/google/firestore/v1/firestore.proto",
    "../../googleapis/google/firestore/admin/v1/firestore_admin.proto",
    // Longrunning
    "../../googleapis/google/longrunning/operations.proto",
    // Monitoring
    "../../googleapis/google/monitoring/v3/alert_service.proto",
    "../../googleapis/google/monitoring/v3/group_service.proto",
    "../../googleapis/google/monitoring/v3/metric_service.proto",
    "../../googleapis/google/monitoring/v3/notification_service.proto",
    "../../googleapis/google/monitoring/v3/query_service.proto",
    "../../googleapis/google/monitoring/v3/service_service.proto",
    "../../googleapis/google/monitoring/v3/service.proto",
    "../../googleapis/google/monitoring/v3/uptime_service.proto",
    // Logging
    "../../googleapis/google/logging/v2/logging.proto",
    "../../googleapis/google/logging/v2/logging_config.proto",
    // Pubsub
    "../../googleapis/google/pubsub/v1/pubsub.proto",
    // Storage V2
    "../../googleapis/google/storage/v2/storage.proto",
    // Cloud Run
    "../../googleapis/google/cloud/run/v2/service.proto",
    "../../googleapis/google/cloud/run/v2/revision.proto",
    // Artifact Registry
    "../../googleapis/google/devtools/artifactregistry/v1/service.proto",
    // Spanner
    "../../googleapis/google/spanner/v1/spanner.proto",
    "../../googleapis/google/spanner/admin/database/v1/spanner_database_admin.proto",
    "../../googleapis/google/spanner/admin/instance/v1/spanner_instance_admin.proto",
    // Cloud Tasks
    "../../googleapis/google/cloud/tasks/v2/cloudtasks.proto",
    "../../googleapis/google/cloud/tasks/v2/queue.proto",
    "../../googleapis/google/cloud/tasks/v2/target.proto",
    "../../googleapis/google/cloud/tasks/v2/task.proto",
];

/*
macro_rules! derive {
    ($($trait:path),* $(,)?) => {{
        derive!("#[derive(", $($trait,)*)
    }};
    ($dst:expr, $leading_trait:path, $($trait:path),+ $(,)?) => {{
        derive!(
            concat!($dst, stringify!($leading_trait), ", "),
            $($trait,)+
        )
    }};
    ($dst:expr, $leading_trait:path $(,)?) => {{
        concat!(
            $dst,
            stringify!($leading_trait),
            ")]"
        )
    }}
}
*/

const EXTRA_TRAITS: &[(&str, &str)] = &[
    //("google.protobuf.Timestamp", derive!(Eq, PartialOrd, Ord)),
    //("google.protobuf.Duration", derive!(Eq, PartialOrd, Ord)),
    //("google.protobuf.DoubleValue", derive!(PartialOrd)),
    //("google.protobuf.FloatValue", derive!(PartialOrd)),
    ("google.protobuf.ListValue", "#[repr(transparent)]"),
    ("google.protobuf.Struct", "#[repr(transparent)]"),
    //("google.protobuf.Int64Value", derive!(Eq, PartialOrd, Ord)),
    //("google.protobuf.UInt64Value", derive!(Eq, PartialOrd, Ord)),
    //("google.protobuf.Int32Value", derive!(Eq, PartialOrd, Ord)),
    //("google.protobuf.UInt32Value", derive!(Eq, PartialOrd, Ord)),
    // ("google.protobuf.BoolValue", derive!(Eq, PartialOrd, Ord)),
    //("google.type.LatLng", derive!(PartialOrd)),
];

#[allow(dead_code)]
const TIMESTAMP_SERDE_ATTR: &str = "#[serde(with = \"crate::impls::timestamp\")]";
#[allow(dead_code)]
const DURATION_SERDE_ATTR: &str = "#[serde(with = \"crate::impls::duration\")]";
#[allow(dead_code)]
const TIMESTAMP_OPT_SERDE_ATTR: &str = "#[serde(with = \"crate::impls::timestamp::opt\")]";

#[allow(dead_code)]
const DURATION_OPT_SERDE_ATTR: &str = "#[serde(with = \"crate::impls::duration::opt\")]";
#[allow(dead_code)]
const SERDE_FLATTEN: &str = "#[serde(flatten)]";
#[allow(dead_code)]
const SERDE_DEFAULT: &str = "#[serde(default)]";

#[allow(unused_macros)]
macro_rules! serde_enum_str {
    ($enum_type:path) => {{
        concat!(
            "#[serde(with = \"crate::impls::SerdeEnumStr::<",
            stringify!($enum_type),
            ">\")]",
        )
    }};
}

fn main() -> std::io::Result<()> {
    // println!("cargo::rerun-if-env-changed=FORCE_BUILD_PROTOS");

    for file in FILES {
        println!("cargo:rerun-if-changed={file}");
    }

    let mut cfg = tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_well_known_types(true)
        .bytes(".")
        .type_attribute(".", "#[derive(serde::Deserialize, serde::Serialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    for (ty, derive) in EXTRA_TRAITS {
        cfg = cfg.type_attribute(ty, derive);
    }

    cfg.out_dir("src/protos").compile_protos(FILES, ROOT)?;

    let status = std::process::Command::new("cargo").arg("fmt").status()?;

    if !status.success() {
        eprintln!("protos build script failed to format generated code");
    }

    Ok(())
}
