//! Main Firestore interface
use std::path::Path;
use std::sync::Arc;

use futures::{Future, Stream};
use gcp_auth_channel::{Auth, AuthChannel, Scope};

use crate::PathComponent;
use crate::batch::read::{BatchRead, Document};
use crate::batch::write::BatchWrite;
use crate::client::FirestoreClient;
use crate::collec::CollectionRef;

const DEFAULT_DATABASE: &str = "(default)";

/// Firestore client for a single database.
#[derive(Clone, Debug)]
pub struct Firestore {
    /// The fully qualified database path, pre-formatted for quick use.
    qualified_db_path: Arc<str>,
    /// The inner auth channel
    pub(crate) client: FirestoreClient,
}

impl PartialEq for Firestore {
    fn eq(&self, other: &Self) -> bool {
        self.qualified_db_path == other.qualified_db_path
    }
}

impl Eq for Firestore {}

impl Firestore {
    #[inline]
    pub fn builder() -> builder::FirestoreBuilder {
        builder::FirestoreBuilder::new()
    }

    /// Initializes a [`Firestore`] client with the default project database.
    pub async fn new(project_id: &'static str, scope: Scope) -> crate::Result<Self> {
        Self::with_database(project_id, DEFAULT_DATABASE, scope).await
    }

    pub fn from_auth_channel(auth_channel: AuthChannel) -> Self {
        let client = FirestoreClient::from_auth_channel(auth_channel);
        Self::from_fs_client(client, DEFAULT_DATABASE)
    }

    pub fn auth(&self) -> &Auth {
        self.client.auth()
    }

    #[inline]
    fn from_fs_client(client: FirestoreClient, db: &str) -> Self {
        let qualified_db_path =
            Arc::from(format!("projects/{}/databases/{db}", client.project_id(),));

        Self {
            client,
            qualified_db_path,
        }
    }

    pub async fn new_with_auth_manager<A>(auth_manager: A) -> crate::Result<Self>
    where
        A: Into<Auth>,
    {
        let auth_manager = auth_manager.into();

        let client = FirestoreClient::from_auth_manager(auth_manager).await?;

        Ok(Self::from_fs_client(client, DEFAULT_DATABASE))
    }

    pub async fn from_auth_manager_future<F>(auth_manager_fut: F) -> crate::Result<Self>
    where
        F: Future,
        F::Output: Into<Auth>,
    {
        let client = FirestoreClient::from_auth_manager_future(auth_manager_fut).await?;

        Ok(Self::from_fs_client(client, DEFAULT_DATABASE))
    }

    pub async fn from_service_account_credentials<P>(
        project_id: &'static str,
        path: P,
        scope: Scope,
    ) -> crate::Result<Self>
    where
        P: AsRef<Path>,
    {
        let client =
            FirestoreClient::from_service_account_credentials(project_id, path, scope).await?;
        Ok(Self::from_fs_client(client, DEFAULT_DATABASE))
    }

    pub(crate) fn qualified_db_path(&self) -> &str {
        &self.qualified_db_path
    }

    /// Initializes a [`Firestore`] client with a given database within the project.
    pub async fn with_database<D>(
        project_id: &'static str,
        database_id: D,
        scope: Scope,
    ) -> crate::Result<Self>
    where
        D: AsRef<str>,
    {
        let client = FirestoreClient::new(project_id, scope).await?;
        Ok(Self::from_fs_client(client, database_id.as_ref()))
    }

    pub fn batch_write(&self) -> BatchWrite {
        BatchWrite::new(self.client.clone(), self.qualified_db_path.clone())
    }

    pub fn transaction(&self) -> crate::transaction::builder::TransactionBuilder {
        crate::transaction::builder::TransactionBuilder::new(self.clone())
    }

    pub fn batch_write_with_capacity(&self, capacity: usize) -> BatchWrite {
        BatchWrite::new_with_write_capacity(
            self.client.clone(),
            self.qualified_db_path.clone(),
            capacity,
        )
    }

    pub fn batch_read(&self) -> BatchRead<Document> {
        BatchRead::new(
            self.client.clone(),
            self.qualified_db_path.clone().into(),
            None,
        )
    }

    pub fn list_collection_ids(&mut self) -> impl Stream<Item = crate::Result<Vec<String>>> + '_ {
        let fully_qualified_path = format!("{}/documents", self.qualified_db_path);
        crate::common::list_collection_ids(&mut self.client, fully_qualified_path)
    }

    /// Returns a reference to a collection.
    pub fn collection<C>(&self, collec_name: C) -> CollectionRef<C>
    where
        C: PathComponent,
    {
        CollectionRef::new_root(collec_name, self.clone())
    }

    /// Returns the ID for the GCP Project this client is running under.
    pub fn project_id(&self) -> &str {
        self.client.project_id()
    }
}

pub mod builder {
    use std::borrow::Cow;
    use std::path::Path;

    use gcp_auth_channel::{Auth, Scope};

    use super::Firestore;
    use crate::client::FirestoreClient;

    pub struct FirestoreBuilder {
        database: Cow<'static, str>,
        scope: Scope,
    }

    impl FirestoreBuilder {
        pub fn new() -> Self {
            Self {
                database: Cow::Borrowed(super::DEFAULT_DATABASE),
                scope: Scope::Firestore,
            }
        }

        pub fn database(&mut self, database: impl Into<Cow<'static, str>>) -> &mut Self {
            self.database = database.into();
            self
        }

        pub fn scope(&mut self, scope: Scope) -> &mut Self {
            self.scope = scope;
            self
        }

        pub async fn from_service_account_credentials(
            &self,
            project_id: &'static str,
            path: impl AsRef<Path>,
        ) -> crate::Result<Firestore> {
            let client = FirestoreClient::from_service_account_credentials(
                project_id,
                path.as_ref(),
                self.scope,
            )
            .await?;

            Ok(Firestore::from_fs_client(client, &self.database))
        }

        pub async fn build(&mut self, project_id: &'static str) -> crate::Result<Firestore> {
            let client = FirestoreClient::new(project_id, self.scope).await?;
            Ok(Firestore::from_fs_client(client, &self.database))
        }

        pub async fn from_auth_manager(&mut self, auth: Auth) -> crate::Result<Firestore> {
            let client = FirestoreClient::from_auth_manager(auth).await?;
            Ok(Firestore::from_fs_client(client, &self.database))
        }

        pub async fn from_auth_manager_future<F>(
            &mut self,
            auth_future: F,
        ) -> crate::Result<Firestore>
        where
            F: Future,
            F::Output: Into<Auth>,
        {
            let client = FirestoreClient::from_auth_manager_future(auth_future).await?;
            Ok(Firestore::from_fs_client(client, &self.database))
        }

        pub async fn from_try_auth_manager_future<F, Error>(
            &mut self,
            auth_future: F,
        ) -> crate::Result<Firestore>
        where
            F: Future<Output = Result<Auth, Error>>,
            crate::Error: From<Error>,
        {
            let client = FirestoreClient::from_try_auth_manager_future(auth_future).await?;
            Ok(Firestore::from_fs_client(client, &self.database))
        }
    }
}
