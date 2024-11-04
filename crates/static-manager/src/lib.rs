#![feature(once_cell, future_join)]

#[cfg(feature = "small-gcs")]
use std::borrow::Cow;

#[cfg(feature = "bigquery-rest")]
pub use bigquery_rs::rest::BigQueryClient;
#[cfg(feature = "bigquery-storage")]
pub use bigquery_rs::storage::BigQueryStorageClient;
#[cfg(feature = "firestore")]
pub use firestore_rs::Firestore;
#[cfg(feature = "auth")]
pub use gcp_auth_channel::Auth;
#[cfg(feature = "pubsub")]
pub use pubsub_rs::PubSubClient;
#[cfg(feature = "realtime-db")]
pub use realtime_database::RealtimeDatabase;
#[cfg(feature = "reqwest")]
pub use reqwest::Client as ReqwestClient;
#[cfg(feature = "small-gcs")]
pub use small_gcs::StorageClient;
use tokio::sync::OnceCell;

#[derive(Debug)]
pub struct StaticManager {
    project_id: &'static str,
    #[cfg(feature = "auth")]
    service_account: Option<&'static str>,
    #[cfg(feature = "auth")]
    auth: OnceCell<Auth>,
    #[cfg(feature = "firestore")]
    firestore: OnceCell<Firestore>,
    #[cfg(feature = "bigquery-storage")]
    bigquery_storage: OnceCell<BigQueryStorageClient>,
    #[cfg(feature = "bigquery-rest")]
    bigquery_rest: OnceCell<BigQueryClient>,
    #[cfg(feature = "realtime-db")]
    realtime_db: OnceCell<RealtimeDatabase>,
    #[cfg(feature = "pubsub")]
    pubsub: OnceCell<PubSubClient>,
    #[cfg(feature = "small-gcs")]
    storage: OnceCell<StorageClient>,
    #[cfg(feature = "reqwest")]
    reqwest: OnceCell<ReqwestClient>,
}

macro_rules! new_uninit {
    ($project_id:ident, $service_account:expr => { $(($name:ident => $($feat:literal),* $(,)?)),* $(,)? }) => {
        StaticManager {
            $project_id,
            #[cfg(feature = "auth")]
            service_account: $service_account,
            $(
                #[cfg(any(
                    $(feature = $feat,)*
                ))]
                $name: OnceCell::const_new(),
            )*
        }
    };
    ($project_id:ident, $service_account:expr) => {
        new_uninit! {
            $project_id, $service_account => {
                (auth => "auth"),
                (firestore => "firestore"),
                (bigquery_storage => "bigquery-storage"),
                (bigquery_rest => "bigquery-rest"),
                (realtime_db => "realtime-db"),
                (pubsub => "pubsub"),
                (storage => "small-gcs"),
                (reqwest => "reqwest"),
            }
        }
    };
}

impl StaticManager {
    pub const fn new(project_id: &'static str) -> Self {
        new_uninit!(project_id, None)
    }

    #[cfg(feature = "auth")]
    pub const fn service_account(project_id: &'static str, cred_file_path: &'static str) -> Self {
        new_uninit!(project_id, Some(cred_file_path))
    }

    #[inline]
    pub fn project_id(&self) -> &'static str {
        self.project_id
    }

    #[cfg(feature = "auth")]
    pub async fn get_auth_manager(&'static self) -> &'static Auth {
        let init_closure = || async {
            tracing::debug!(
                message = "initializing auth manager",
                service_account = tracing::field::debug(&self.service_account),
            );

            let manager = match self.service_account {
                Some(path) => Auth::new_from_service_account_file(
                    self.project_id,
                    path,
                    gcp_auth_channel::Scope::CloudPlatformAdmin,
                ),
                None => {
                    Auth::new(self.project_id, gcp_auth_channel::Scope::CloudPlatformAdmin).await
                }
            };

            manager.expect("could not build auth manager")
        };

        self.auth.get_or_init(init_closure).await
    }

    #[cfg(feature = "auth")]
    pub async fn get_shared_auth_manager(&'static self) -> Auth {
        self.get_auth_manager().await.clone()
    }

    #[cfg(feature = "firestore")]
    pub async fn get_firestore(&'static self) -> &'static Firestore {
        let init_closure = || async {
            let auth_manager = self.get_shared_auth_manager().await;
            tracing::debug!(message = "initializing firestore");

            Firestore::new_with_auth_manager(auth_manager)
                .await
                .expect("could not build firestore client")
        };

        self.firestore.get_or_init(init_closure).await
    }

    #[cfg(feature = "bigquery-storage")]
    pub async fn get_bigquery_storage(&'static self) -> &'static BigQueryStorageClient {
        let init_closure = || async {
            let auth_manager = self.get_shared_auth_manager().await;
            tracing::debug!(message = "initializing bigquery storage");

            BigQueryStorageClient::from_auth_manager(self.project_id, auth_manager)
                .await
                .expect("could not build the bigquery storage client")
        };

        self.bigquery_storage.get_or_init(init_closure).await
    }
    #[cfg(feature = "bigquery-rest")]
    pub async fn get_bigquery_rest(&'static self) -> &'static BigQueryClient {
        let init_closure = || async {
            let _auth_manager = self.get_shared_auth_manager().await;
            tracing::debug!(message = "initializing bigquery storage");

            bigquery_rs::rest::BigQueryClient::new(self.project_id)
                .await
                .expect("could not build bigquery rest client")
        };

        self.bigquery_rest.get_or_init(init_closure).await
    }

    #[cfg(feature = "realtime-db")]
    pub async fn get_realtime_db(&'static self) -> &'static RealtimeDatabase {
        let init_closure = || async {
            let auth_manager = self.get_shared_auth_manager().await;
            tracing::debug!(message = "initializing realtime database client");

            RealtimeDatabase::builder()
                .project_id(self.project_id)
                .with_auth_manager(auth_manager)
                .build()
                .await
                .expect("could not build realtime database client")
        };

        self.realtime_db.get_or_init(init_closure).await
    }

    #[cfg(feature = "pubsub")]
    pub async fn get_pubsub(&'static self) -> &'static PubSubClient {
        let init_closure = || async {
            let auth_manager = self.get_shared_auth_manager().await;
            tracing::debug!(message = "initializing pubsub client");

            PubSubClient::from_auth_manager(auth_manager)
                .await
                .expect("could not build pubsub client")
        };

        self.pubsub.get_or_init(init_closure).await
    }

    #[cfg(feature = "small-gcs")]
    async fn get_storage_parts(&'static self) -> (reqwest::Client, Auth) {
        let (client, auth) = tokio::join!(self.get_reqwest(), self.get_shared_auth_manager());
        (client.clone(), auth)
    }

    #[cfg(feature = "small-gcs")]
    pub async fn get_default_storage(
        &'static self,
        bucket_name: &'static str,
    ) -> Option<&'static StorageClient> {
        /// Need to keep track of the bucket so we can make a new client on demand if needed.
        static DEFAULT_BUCKET: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();

        // first thing first, set the default bucket if it's unset.
        if DEFAULT_BUCKET.get().is_none() {
            let _ = DEFAULT_BUCKET.set(bucket_name);
        }

        // then, check if the value in there matches the passed in bucket name
        let default_bucket = DEFAULT_BUCKET
            .get()
            .copied()
            .expect("set was called, should be initialized");

        if bucket_name == default_bucket {
            let init_closure = || async {
                let (client, auth) = self.get_storage_parts().await;
                StorageClient::from_parts(client, auth, default_bucket.into())
            };

            Some(self.storage.get_or_init(init_closure).await)
        } else {
            None
        }
    }

    #[cfg(feature = "small-gcs")]
    pub async fn get_storage(
        &'static self,
        bucket_name: &'static str,
    ) -> Cow<'static, StorageClient> {
        match self.get_default_storage(bucket_name).await {
            Some(default) => Cow::Borrowed(default),
            None => {
                let (client, auth) = self.get_storage_parts().await;
                Cow::Owned(StorageClient::from_parts(client, auth, bucket_name.into()))
            }
        }
    }

    #[cfg(feature = "reqwest")]
    pub async fn get_reqwest(&'static self) -> &'static ReqwestClient {
        let init_closure = || async {
            ReqwestClient::builder()
                .user_agent("mysticetus-rs")
                .build()
                .expect("could not build reqwest client")
        };

        self.reqwest.get_or_init(init_closure).await
    }
}
