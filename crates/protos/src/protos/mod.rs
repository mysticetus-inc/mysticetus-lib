#[path = ""]
pub(super) mod google {

    #[cfg(feature = "api")]
    #[path = "google.api.rs"]
    pub mod api;

    #[cfg(feature = "protobuf")]
    #[path = "google.protobuf.rs"]
    pub mod protobuf;

    #[cfg(feature = "rpc")]
    #[path = "google.rpc.rs"]
    pub mod rpc;

    #[cfg(feature = "longrunning")]
    #[path = "google.longrunning.rs"]
    pub mod longrunning;

    #[cfg(feature = "type")]
    #[path = "google.r#type.rs"]
    pub mod r#type;

    #[cfg(feature = "monitoring")]
    #[path = ""]
    pub mod monitoring {
        #[path = "google.monitoring.v3.rs"]
        pub mod v3;
    }

    #[cfg(feature = "pubsub")]
    #[path = ""]
    pub mod pubsub {
        #[path = "google.pubsub.v1.rs"]
        pub mod v1;
    }

    #[cfg(feature = "iam")]
    #[path = ""]
    pub mod iam {
        #[path = "google.iam.v1.rs"]
        pub mod v1;
    }

    #[cfg(feature = "storage")]
    #[path = ""]
    pub mod storage {
        #[path = "google.storage.v2.rs"]
        pub mod v2;
    }

    #[cfg(feature = "logging")]
    #[path = ""]
    pub mod logging {
        #[path = "google.logging.r#type.rs"]
        pub mod r#type;

        #[path = "google.logging.v2.rs"]
        pub mod v2;
    }

    #[cfg(feature = "firestore")]
    #[path = ""]
    pub mod firestore {
        #[path = "google.firestore.v1.rs"]
        pub mod v1;

        #[cfg(feature = "firestore-admin")]
        #[path = ""]
        pub mod admin {
            #[path = "google.firestore.admin.v1.rs"]
            pub mod v1;
        }
    }

    #[cfg(any(feature = "bigquery", feature = "cloud-run"))]
    #[path = ""]
    pub mod cloud {

        #[cfg(feature = "cloud-run")]
        #[path = ""]
        pub mod run {
            #[path = "google.cloud.run.v2.rs"]
            pub mod v2;
        }

        #[cfg(feature = "bigquery")]
        #[path = ""]
        pub mod bigquery {
            #[path = "google.cloud.bigquery.v2.rs"]
            pub mod v2;

            #[path = ""]
            pub mod storage {

                #[path = "google.cloud.bigquery.storage.v1.rs"]
                pub mod v1;
            }
        }
    }

    #[cfg(feature = "artifact-registry")]
    #[path = ""]
    pub mod devtools {
        #[path = ""]
        pub mod artifactregistry {
            #[path = "google.devtools.artifactregistry.v1.rs"]
            pub mod v1;
        }
    }

    #[cfg(any(
        feature = "spanner",
        feature = "spanner-admin-database",
        feature = "spanner-admin-instance",
    ))]
    #[path = ""]
    pub mod spanner {
        #[cfg(feature = "spanner")]
        #[path = "google.spanner.v1.rs"]
        pub mod v1;

        #[cfg(any(feature = "spanner-admin-database", feature = "spanner-admin-instance"))]
        #[path = ""]
        pub mod admin {
            #[cfg(feature = "spanner-admin-database")]
            #[path = ""]
            pub mod database {
                #[path = "google.spanner.admin.database.v1.rs"]
                pub mod v1;
            }

            #[cfg(feature = "spanner-admin-instance")]
            #[path = ""]
            pub mod instance {
                #[path = "google.spanner.admin.instance.v1.rs"]
                pub mod v1;
            }
        }
    }
}
/*

#[cfg(any(
    feature = "mysticetus-common",
    feature = "mysticetus-config",
    feature = "mysticetus-validation",
    feature = "mysticetus-video",
))]
#[path = ""]
pub mod mysticetus {
    #[cfg(feature = "mysticetus-common")]
    #[path = "mysticetus.common.rs"]
    pub mod common;

    #[cfg(feature = "mysticetus-config")]
    #[path = "mysticetus.config.rs"]
    pub mod config;

    #[cfg(feature = "mysticetus-validation")]
    #[path = "mysticetus.validation.rs"]
    pub mod validation;

    #[cfg(feature = "mysticetus-video")]
    #[path = "mysticetus.video.rs"]
    pub mod video;

    #[cfg(feature = "mysticetus-video-rpc")]
    #[path = ""]
    pub mod rpc {
        #[cfg(feature = "mysticetus-video-rpc")]
        #[path = "mysticetus.rpc.video.rs"]
        pub mod video;
    }
}
*/
