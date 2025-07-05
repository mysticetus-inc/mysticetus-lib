#[cfg(feature = "application-default")]
mod application_default;
mod client;
#[cfg(feature = "emulator")]
pub mod emulator;
mod error;
mod future;
#[cfg(feature = "gcloud")]
pub mod gcloud;
pub mod metadata;
mod project_id;
mod scope;
pub mod service_account;
mod token;
mod util;

pub use error::{Error, ResponseError};
pub use future::GetTokenFuture;
pub use project_id::ProjectId;
pub use scope::{Scope, Scopes};
pub use token::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Provider {
    #[cfg(feature = "application-default")]
    ApplicationDefault(application_default::ApplicationDefault),
    #[cfg(feature = "emulator")]
    Emulator(emulator::EmulatorProvider),
    #[cfg(feature = "gcloud")]
    GCloud(gcloud::GCloudProvider),
    MetadataServer(metadata::MetadataServer),
    ServiceAccount(service_account::ServiceAccount),
}

impl Provider {
    #[cfg(feature = "emulator")]
    pub const fn new_emulator() -> Self {
        Self::Emulator(emulator::EmulatorProvider)
    }

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

        #[cfg(feature = "application-default")]
        if let Some((app, project_id)) =
            application_default::ApplicationDefault::try_load(&mut ctx).await?
        {
            return Ok((Self::ApplicationDefault(app), project_id));
        }

        #[cfg(feature = "gcloud")]
        if let Some((gcloud, project_id)) = gcloud::GCloudProvider::try_load().await? {
            return Ok((Self::GCloud(gcloud), project_id));
        }

        Err(Error::NoProviderFound)
    }

    pub fn token_provider_name(&self) -> &'static str {
        match self {
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(app) => app.name(),
            #[cfg(feature = "emulator")]
            Self::Emulator(emulator) => emulator.name(),
            #[cfg(feature = "gcloud")]
            Self::GCloud(gcloud) => gcloud.name(),
            Self::MetadataServer(ms) => ms.name(),
            Self::ServiceAccount(svc) => svc.name(),
        }
    }

    pub fn as_token_provider(&self) -> Option<&dyn TokenProvider> {
        match self {
            #[cfg(feature = "gcloud")]
            Self::GCloud(gcloud) => Some(gcloud),
            #[cfg(feature = "emulator")]
            Self::Emulator(emulator) => Some(emulator),
            Self::MetadataServer(meta) => Some(meta),
            _ => None,
        }
    }

    pub fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        match self {
            #[cfg(feature = "application-default")]
            Self::ApplicationDefault(app) => app.get_scoped_token(scopes),
            #[cfg(feature = "gcloud")]
            Self::GCloud(gcloud) => gcloud.get_token(),
            #[cfg(feature = "emulator")]
            Self::Emulator(emulator) => emulator.get_token(),
            Self::MetadataServer(meta) => meta.get_token(),
            Self::ServiceAccount(acct) => acct.get_scoped_token(scopes),
        }
    }
}

/// Base trait for token providers. Actual provider impls
/// are defined by the supertraits [`ScopedTokenProvider`] and [`TokenProvider`].
pub trait BaseTokenProvider: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;
}

impl<B: BaseTokenProvider> BaseTokenProvider for &B {
    #[inline]
    fn name(&self) -> &'static str {
        B::name(self)
    }
}

impl BaseTokenProvider for Provider {
    fn name(&self) -> &'static str {
        self.token_provider_name()
    }
}

impl ScopedTokenProvider for Provider {
    #[inline]
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        self.get_scoped_token(scopes)
    }
}

pub trait ScopedTokenProvider: BaseTokenProvider {
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_>;

    fn into_scoped(self, scopes: Scopes) -> ScopedProvider<Self>
    where
        Self: Sized,
    {
        ScopedProvider {
            provider: self,
            scopes,
        }
    }
}

impl<S> ScopedTokenProvider for &S
where
    S: ScopedTokenProvider,
{
    #[inline]
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        S::get_scoped_token(self, scopes)
    }
}

fn _assert_scoped_token_provider_dyn(_: &dyn ScopedTokenProvider) {}

/// For providers that don't need any scopes to generate a token.
pub trait TokenProvider: BaseTokenProvider {
    fn get_token(&self) -> GetTokenFuture<'_>;
}

impl<S> TokenProvider for &S
where
    S: TokenProvider,
{
    #[inline]
    fn get_token(&self) -> GetTokenFuture<'_> {
        <S as TokenProvider>::get_token(self)
    }
}

fn _assert_token_provider_dyn(_: &dyn TokenProvider) {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopedProvider<P> {
    provider: P,
    scopes: Scopes,
}

impl<B: BaseTokenProvider> BaseTokenProvider for ScopedProvider<B> {
    #[inline]
    fn name(&self) -> &'static str {
        self.provider.name()
    }
}

impl<P: ScopedTokenProvider> TokenProvider for ScopedProvider<P> {
    #[inline]
    fn get_token(&self) -> GetTokenFuture<'_> {
        self.provider.get_scoped_token(self.scopes)
    }
}

#[derive(Debug, Default)]
struct InitContext {
    http: Option<client::HttpClient>,
    https: Option<client::HttpsClient>,
}
