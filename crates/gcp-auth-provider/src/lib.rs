mod application_default;
mod client;
mod error;
#[cfg(feature = "gcloud")]
mod gcloud;
mod metadata;
mod scope;
mod service_account;
mod state;
mod token;
mod util;
use std::future::Future;
use std::sync::Arc;

pub use error::{Error, ResponseError};
pub use scope::{Scope, Scopes};
pub use token::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum TokenProvider {
    #[cfg(feature = "gcloud")]
    GCloud(gcloud::GCloudProvider),
    MetadataServer(Arc<metadata::MetadataServer>),
    ServiceAccount(Arc<service_account::ServiceAccount>),
    ApplicationDefault(Arc<application_default::ApplicationDefault>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectId(pub Arc<str>);

impl TokenProvider {
    pub async fn detect() -> Result<(Self, ProjectId)> {
        let mut ctx = InitContext::default();

        if let Some((service_account, project_id)) =
            service_account::ServiceAccount::try_load(&mut ctx).await?
        {
            return Ok((Self::ServiceAccount(Arc::new(service_account)), project_id));
        }

        if let Some((metadata, project_id)) = metadata::MetadataServer::try_load(&mut ctx).await? {
            return Ok((Self::MetadataServer(Arc::new(metadata)), project_id));
        }

        if let Some((app, project_id)) =
            application_default::ApplicationDefault::try_load(&mut ctx).await?
        {
            return Ok((Self::ApplicationDefault(Arc::new(app)), project_id));
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
        &self,
        scopes: Scopes,
    ) -> impl Future<Output = Result<Token>> + Send + 'static {
        let this = self.clone();
        async move {
            match this {
                #[cfg(feature = "gcloud")]
                Self::GCloud(gcloud) => gcloud.get_token(scopes).await,
                Self::MetadataServer(meta) => meta.get_token(scopes).await,
                Self::ServiceAccount(acct) => acct.get_token(scopes).await,
                Self::ApplicationDefault(app) => app.get_token(scopes).await,
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
