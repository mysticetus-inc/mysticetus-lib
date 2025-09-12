//! Main Firestore interface
use std::path::PathBuf;

use futures::{Future, Stream};
use gcp_auth_provider::service::AuthSvc;
use gcp_auth_provider::{Auth, Scope};
use tonic::transport::Channel;

use crate::PathComponent;
use crate::batch::read::{BatchRead, Document};
use crate::batch::write::BatchWrite;
use crate::client::FirestoreClient;
use crate::collec::CollectionRef;

const DEFAULT_DATABASE: &str = "(default)";

/// Firestore client for a single database.
#[derive(Clone, Debug)]
pub struct Firestore {
    /// The inner auth channel
    pub(crate) client: FirestoreClient,
}

impl PartialEq for Firestore {
    fn eq(&self, other: &Self) -> bool {
        str::eq(
            &*self.client.qualified_db_path,
            &*other.client.qualified_db_path,
        )
    }
}

impl Eq for Firestore {}

impl Firestore {
    #[inline]
    pub fn builder<'a>() -> builder::FirestoreBuilder<'a> {
        builder::FirestoreBuilder::new()
    }

    /// Initializes a [`Firestore`] client with the default project database.
    pub async fn new(scope: Scope) -> crate::Result<Self> {
        Self::with_database(DEFAULT_DATABASE, scope).await
    }

    pub fn from_auth_channel(auth_channel: AuthSvc<Channel>) -> Self {
        let client = FirestoreClient::from_auth_svc(auth_channel, DEFAULT_DATABASE);
        Self { client }
    }

    pub fn auth(&self) -> &Auth {
        self.client.auth()
    }

    pub async fn new_with_auth_manager<A>(auth_manager: A) -> crate::Result<Self>
    where
        A: Into<Auth>,
    {
        let auth_manager = auth_manager.into();

        let client = FirestoreClient::from_auth_manager(auth_manager, DEFAULT_DATABASE).await?;

        Ok(Self { client })
    }

    pub async fn from_auth_manager_future<F>(auth_manager_fut: F) -> crate::Result<Self>
    where
        F: Future,
        F::Output: Into<Auth>,
    {
        let client =
            FirestoreClient::from_auth_manager_future(auth_manager_fut, DEFAULT_DATABASE).await?;

        Ok(Self { client })
    }

    pub async fn from_service_account_credentials<P>(path: P, scope: Scope) -> crate::Result<Self>
    where
        P: Into<PathBuf>,
    {
        let client = FirestoreClient::from_service_account_credentials::<PathBuf>(
            path.into(),
            scope,
            DEFAULT_DATABASE,
        )
        .await?;
        Ok(Self { client })
    }

    pub(crate) fn qualified_db_path(&self) -> &str {
        &self.client.qualified_db_path
    }

    /// Initializes a [`Firestore`] client with a given database within the project.
    pub async fn with_database<D>(database_id: D, scope: Scope) -> crate::Result<Self>
    where
        D: AsRef<str>,
    {
        let client = FirestoreClient::new(scope, database_id.as_ref()).await?;
        Ok(Self { client })
    }

    pub fn batch_write(&self) -> BatchWrite {
        BatchWrite::new(self.client.clone())
    }

    pub fn transaction(&self) -> crate::transaction::builder::TransactionBuilder {
        crate::transaction::builder::TransactionBuilder::new(self.clone())
    }

    pub fn batch_write_with_capacity(&self, capacity: usize) -> BatchWrite {
        BatchWrite::new_with_write_capacity(self.client.clone(), capacity)
    }

    pub fn batch_read(&self) -> BatchRead<Document> {
        BatchRead::new(
            self.client.clone(),
            From::from(self.client.qualified_db_path.clone()),
            None,
        )
    }

    pub fn list_collection_ids(&mut self) -> impl Stream<Item = crate::Result<Vec<String>>> + '_ {
        let fully_qualified_path = format!("{}/documents", self.client.qualified_db_path);
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
    use std::path::PathBuf;

    use gcp_auth_provider::{Auth, Scope};

    use super::Firestore;
    use crate::client::FirestoreClient;

    pub struct FirestoreBuilder<'a> {
        database: Cow<'a, str>,
        scope: Scope,
    }

    impl<'a> FirestoreBuilder<'a> {
        pub fn new() -> Self {
            Self {
                database: Cow::Borrowed(super::DEFAULT_DATABASE),
                scope: Scope::Firestore,
            }
        }

        pub fn database(&mut self, database: impl Into<Cow<'a, str>>) -> &mut Self {
            self.database = database.into();
            self
        }

        pub fn scope(&mut self, scope: Scope) -> &mut Self {
            self.scope = scope;
            self
        }

        pub async fn from_service_account_credentials(
            &self,
            path: impl Into<PathBuf>,
        ) -> crate::Result<Firestore> {
            let client = FirestoreClient::from_service_account_credentials(
                path.into(),
                self.scope,
                &self.database,
            )
            .await?;

            Ok(Firestore { client })
        }

        pub async fn build(&mut self) -> crate::Result<Firestore> {
            let client = FirestoreClient::new(self.scope, &self.database).await?;
            Ok(Firestore { client })
        }

        pub async fn from_auth_manager(&mut self, auth: Auth) -> crate::Result<Firestore> {
            let client = FirestoreClient::from_auth_manager(auth, &self.database).await?;
            Ok(Firestore { client })
        }

        pub async fn from_auth_manager_future<F>(
            &mut self,
            auth_future: F,
        ) -> crate::Result<Firestore>
        where
            F: Future,
            F::Output: Into<Auth>,
        {
            let client =
                FirestoreClient::from_auth_manager_future(auth_future, &self.database).await?;
            Ok(Firestore { client })
        }

        pub async fn from_try_auth_manager_future<F, Error>(
            &mut self,
            auth_future: F,
        ) -> crate::Result<Firestore>
        where
            F: Future<Output = Result<Auth, Error>>,
            crate::Error: From<Error>,
        {
            let client =
                FirestoreClient::from_try_auth_manager_future(auth_future, &self.database).await?;
            Ok(Firestore { client })
        }
    }
}
