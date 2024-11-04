//! Main Firestore interface
use std::path::Path;
use std::sync::Arc;

use futures::{Future, Stream};
use gcp_auth_channel::{Auth, AuthChannel, Scope};

use crate::batch::read::{BatchRead, Document};
use crate::batch::write::BatchWrite;
use crate::client::FirestoreClient;
use crate::collec::CollectionRef;
use crate::PathComponent;

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
    /// Initializes a [`Firestore`] client with the default project database.
    pub async fn new(project_id: &'static str, scope: Scope) -> crate::Result<Self> {
        Self::with_database(project_id, "(default)", scope).await
    }

    pub fn from_auth_channel(auth_channel: AuthChannel) -> Self {
        let client = FirestoreClient::from_auth_channel(auth_channel);
        Self::from_fs_client(client, None)
    }

    pub fn auth(&self) -> &Auth {
        self.client.auth()
    }

    #[inline]
    fn from_fs_client(client: FirestoreClient, db: Option<&str>) -> Self {
        let qualified_db_path = Arc::from(format!(
            "projects/{}/databases/{}",
            client.project_id(),
            db.unwrap_or("(default)")
        ));

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

        Ok(Self::from_fs_client(client, None))
    }

    pub async fn from_auth_manager_future<F>(auth_manager_fut: F) -> crate::Result<Self>
    where
        F: Future,
        F::Output: Into<Auth>,
    {
        let client = FirestoreClient::from_auth_manager_future(auth_manager_fut).await?;

        Ok(Self::from_fs_client(client, None))
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
        Ok(Self::from_fs_client(client, None))
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
        Ok(Self::from_fs_client(client, Some(database_id.as_ref())))
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
