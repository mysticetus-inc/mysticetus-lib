#![cfg_attr(feature = "gcloud", feature(exit_status_error))]

mod cache;
mod client;
mod error;
mod project_id;
pub mod scope;
mod token;
mod util;

#[cfg(feature = "channel")]
pub mod channel;
pub mod providers;
pub mod service;

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

pub use cache::GetHeaderResult;
pub use error::{Error, ResponseError};
use futures::FutureExt;
pub use project_id::ProjectId;
pub use providers::{DetectFuture, GetTokenFuture, Provider};
pub use scope::{Scope, Scopes};
pub use token::Token;

pub use crate::providers::TokenProvider;
use crate::providers::{LoadProviderResult, ScopedTokenProvider, UnscopedProvider};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone)]
pub struct Auth {
    inner: Arc<dyn cache::TokenCache>,
}

pin_project_lite::pin_project! {
    pub struct AuthDetectFuture {
        #[pin]
        fut: futures::future::Map<
            DetectFuture<'static>,
            fn(<DetectFuture<'static> as Future>::Output) -> Result<std::result::Result<Auth, UnscopedProvider>>,
        >,
    }
}

impl AuthDetectFuture {
    pub fn cloud_platform_admin(self) -> ScopedAuthDetectFuture {
        self.with_scopes(Scopes::CLOUD_PLATFORM_ADMIN)
    }

    pub fn cloud_platform_read_only(self) -> ScopedAuthDetectFuture {
        self.with_scopes(Scopes::CLOUD_PLATFORM_READ_ONLY)
    }

    pub fn with_scopes(self, scopes: impl Into<Scopes>) -> ScopedAuthDetectFuture {
        ScopedAuthDetectFuture {
            fut: self.fut,
            scopes: scopes.into(),
        }
    }
}

impl Future for AuthDetectFuture {
    type Output = Result<std::result::Result<Auth, UnscopedProvider>>;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.project().fut.poll(cx)
    }
}

pin_project_lite::pin_project! {
    pub struct ScopedAuthDetectFuture {
        #[pin]
        fut: futures::future::Map<
            DetectFuture<'static>,
            fn(<DetectFuture<'static> as Future>::Output) -> Result<std::result::Result<Auth, UnscopedProvider>>,
        >,
        scopes: Scopes,
    }
}

impl Future for ScopedAuthDetectFuture {
    type Output = Result<Auth>;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        std::task::Poll::Ready(Ok(match std::task::ready!(this.fut.poll(cx))? {
            Ok(prov) => prov,
            Err(unscoped) => unscoped.with_scope(*this.scopes),
        }))
    }
}

pub type AuthMetadataFuture = futures::future::Map<
    providers::metadata::LoadFuture<'static>,
    fn(<providers::metadata::LoadFuture<'static> as Future>::Output) -> Result<Auth>,
>;

pin_project_lite::pin_project! {
    pub struct AuthServiceAccountFuture {
        #[pin]
        inner: providers::service_account::TryLoadFuture<'static>,
        scopes: Scopes,
    }
}
impl Future for AuthServiceAccountFuture {
    type Output = Result<Auth>;

    #[inline]
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let load_result = std::task::ready!(this.inner.poll(cx))?;

        let mapped_result = load_result.map_provider(|provider| provider.with_scopes(*this.scopes));

        std::task::Poll::Ready(Ok(Auth::new_from_provider(mapped_result)))
    }
}

impl Auth {
    #[cfg(feature = "channel")]
    pub fn builder() -> channel::AuthChannelBuilder<(), ()> {
        channel::AuthChannelBuilder::default()
    }

    #[cfg(feature = "channel")]
    pub fn build_channel(&self) -> channel::AuthChannelBuilder<Auth, ()> {
        channel::AuthChannelBuilder::default().auth(self.clone())
    }

    pub fn get_header(&self) -> cache::GetHeaderResult<'_> {
        cache::GetHeaderResult::get(&self.inner)
    }

    /// Identical to calling [tower::Layer<Svc>::layer], but takes
    /// and owned Auth, avoiding an extra clone.
    #[inline]
    pub fn into_service<Svc>(self, svc: Svc) -> service::AuthSvc<Svc> {
        service::AuthSvc { svc, auth: self }
    }

    pub fn project_id(&self) -> ProjectId {
        self.inner.project_id()
    }

    pub fn revoke_token(&self) {
        self.inner.revoke(cache::StartNewRequestOnRevoke::Yes);
    }

    pub fn new_detect() -> AuthDetectFuture {
        AuthDetectFuture {
            fut: Provider::detect().map(Self::from_any_provider_res),
        }
    }

    pub fn new_metadata() -> AuthMetadataFuture {
        providers::metadata::MetadataServer::new()
            .start_token_request()
            .map(|result| result.map(Self::new_from_provider))
    }

    pub fn from_service_account_file(
        path: impl Into<PathBuf>,
        scopes: impl Into<Scopes>,
    ) -> AuthServiceAccountFuture {
        let inner = providers::service_account::ServiceAccount::new_from_path(path.into());

        AuthServiceAccountFuture {
            inner,
            scopes: scopes.into(),
        }
    }

    fn from_any_provider(
        res: LoadProviderResult<'static, Provider>,
    ) -> std::result::Result<Self, UnscopedProvider> {
        let LoadProviderResult {
            provider,
            project_id,
            token_future,
        } = res;

        match provider {
            Provider::MetadataServer(metadata) => Ok(Self::new_from_provider(LoadProviderResult {
                provider: metadata,
                project_id,
                token_future,
            })),
            Provider::ServiceAccount(svc) => Err(UnscopedProvider::ServiceAccount(project_id, svc)),
            #[cfg(feature = "gcloud")]
            Provider::GCloud(gcloud) => Ok(Self::new_from_provider(LoadProviderResult {
                provider: gcloud,
                project_id,
                token_future,
            })),
            #[cfg(feature = "application-default")]
            Provider::ApplicationDefault(app_def) => {
                Err(UnscopedProvider::ApplicationDefault(project_id, app_def))
            }
            #[cfg(feature = "emulator")]
            Provider::Emulator(emulator) => Ok(Self::new_from_provider(LoadProviderResult {
                provider: emulator,
                project_id,
                token_future,
            })),
        }
    }

    fn from_any_provider_res(
        result: Result<LoadProviderResult<'static, Provider>>,
    ) -> Result<std::result::Result<Self, UnscopedProvider>> {
        result.map(Self::from_any_provider)
    }

    pub fn new_from_provider<T: TokenProvider + 'static>(
        load_res: LoadProviderResult<'static, T>,
    ) -> Self {
        let LoadProviderResult {
            provider,
            project_id,
            token_future,
        } = load_res;

        Self {
            inner: Arc::new(cache::CachedTokenProvider::new(
                provider,
                project_id,
                token_future,
            )),
        }
    }
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Auth")
            .field("project_id", &self.inner.project_id())
            .field("provider", &self.inner.provider_name())
            .field("cache", &self.inner)
            .finish()
    }
}

impl PartialEq for Auth {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
            || (self.inner.provider_name() == other.inner.provider_name()
                && self.inner.project_id() == other.inner.project_id())
    }
}

impl<Svc> tower::Layer<Svc> for Auth {
    type Service = service::AuthSvc<Svc>;

    fn layer(&self, svc: Svc) -> Self::Service {
        service::AuthSvc {
            auth: self.clone(),
            svc,
        }
    }
}

pin_project_lite::pin_project! {
    pub struct SharedAuthFuture<F> {
        #[pin]
        inner: Arc<parking_lot::Mutex<SharedInner<F>>>,
    }
}

impl<F> Clone for SharedAuthFuture<F> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = SharedInnerProjection]
    enum SharedInner<F> {
        Error,
        Done { auth: Auth, },
        Pending {
            #[pin]
            fut: F,
        }
    }
}

impl<F> Future for SharedAuthFuture<F>
where
    F: Future<Output = Result<Auth>> + Unpin,
{
    type Output = Result<Auth>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        let mut guard = this.inner.lock();

        loop {
            match Pin::new(&mut *guard).project() {
                SharedInnerProjection::Error => todo!(),
                SharedInnerProjection::Done { auth } => {
                    return std::task::Poll::Ready(Ok(auth.clone()));
                }
                SharedInnerProjection::Pending { fut } => match std::task::ready!(fut.poll(cx)) {
                    Ok(auth) => {
                        *guard = SharedInner::Done { auth: auth.clone() };
                        return std::task::Poll::Ready(Ok(auth));
                    }
                    Err(error) => {
                        *guard = SharedInner::Error;
                        return std::task::Poll::Ready(Err(error));
                    }
                },
            }
        }
    }
}
