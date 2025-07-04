mod application_default;
mod client;
mod error;
#[cfg(feature = "gcloud")]
pub mod gcloud;
pub mod metadata;
mod scope;
pub mod service_account;
mod state;
mod token;
mod util;
use std::future::Future;
use std::sync::Arc;

pub use error::{Error, ResponseError};
pub use scope::{Scope, Scopes};
pub use token::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum TokenProvider {
    #[cfg(feature = "gcloud")]
    GCloud(gcloud::GCloudProvider),
    MetadataServer(metadata::MetadataServer),
    ServiceAccount(service_account::ServiceAccount),
    ApplicationDefault(application_default::ApplicationDefault),
}

#[derive(Debug, Clone)]
pub struct ProjectId(shared::Shared<str>);

impl<S> From<S> for ProjectId
where
    shared::Shared<str>: From<S>,
{
    #[inline]
    fn from(value: S) -> Self {
        Self(From::from(value))
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl<S: AsRef<str>> PartialEq<S> for ProjectId {
    #[inline]
    fn eq(&self, other: &S) -> bool {
        str::eq(self, other.as_ref())
    }
}

impl Eq for ProjectId {}

impl<S: AsRef<str>> PartialOrd<S> for ProjectId {
    #[inline]
    fn partial_cmp(&self, other: &S) -> Option<std::cmp::Ordering> {
        str::partial_cmp(self, other.as_ref())
    }
}

impl Ord for ProjectId {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        str::cmp(self, other)
    }
}

impl std::borrow::Borrow<str> for ProjectId {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl std::ops::Deref for ProjectId {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ProjectId {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TokenProvider {
    pub async fn detect() -> Result<(Self, ProjectId)> {
        let mut ctx = InitContext::default();

        if let Some((service_account, project_id)) =
            service_account::ServiceAccount::try_load(&mut ctx).await?
        {
            return Ok((Self::ServiceAccount(service_account), project_id));
        }

        if let Some((metadata, project_id)) = metadata::MetadataServer::try_load(&mut ctx).await? {
            return Ok((Self::MetadataServer(metadata), project_id));
        }

        if let Some((app, project_id)) =
            application_default::ApplicationDefault::try_load(&mut ctx).await?
        {
            return Ok((Self::ApplicationDefault(app), project_id));
        }

        #[cfg(feature = "gcloud")]
        if let Some((gcloud, project_id)) = gcloud::GCloudProvider::try_load(&mut ctx).await? {
            return Ok((Self::GCloud(gcloud), project_id));
        }

        Err(Error::NoProviderFound)
    }

    pub fn token_provider_name(&self) -> &'static str {
        match self {
            #[cfg(feature = "gcloud")]
            Self::GCloud(_) => gcloud::GCloud::NAME,
            Self::MetadataServer(_) => metadata::MetadataServer::NAME,
            Self::ServiceAccount(_) => service_account::ServiceAccount::NAME,
            Self::ApplicationDefault(_) => application_default::ApplicationDefault::NAME,
        }
    }

    pub fn get_token(
        self: &Arc<Self>,
        scopes: Scopes,
    ) -> impl Future<Output = Result<Token>> + Send + 'static {
        let this = Arc::clone(self);
        async move {
            match *this {
                #[cfg(feature = "gcloud")]
                Self::GCloud(ref gcloud) => gcloud.get_token(scopes).await,
                Self::MetadataServer(ref meta) => meta.get_token(scopes).await,
                Self::ServiceAccount(ref acct) => acct.get_token(scopes).await,
                Self::ApplicationDefault(ref app) => app.get_token(scopes).await,
            }
        }
    }
}

trait RawTokenProvider: std::fmt::Debug + Sized + Send + Sync + 'static {
    const NAME: &'static str;

    fn try_load(
        init_ctx: &mut InitContext,
    ) -> impl Future<Output = Result<Option<(Self, ProjectId)>>> + Send + '_;

    fn get_token(&self, scopes: Scopes) -> impl Future<Output = Result<Token>> + Send + 'static;
}

#[derive(Debug, Default)]
struct InitContext {
    http: Option<client::HttpClient>,
    https: Option<client::HttpsClient>,
}
