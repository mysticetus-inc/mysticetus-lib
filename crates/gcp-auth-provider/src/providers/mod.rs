//! Contains the raw providers, with __no__ token reuse/caching.
//!
//! Shouldn't be used for actual authentication directly.
#[cfg(feature = "application-default")]
mod application_default;
#[cfg(feature = "emulator")]
pub mod emulator;
#[cfg(feature = "gcloud")]
pub mod gcloud;
pub mod metadata;
pub mod service_account;

mod future;
mod provider;

pub use future::GetTokenFuture;
use futures::TryFuture;
use futures::future::TryMaybeDone;
use provider::InitContext;
pub use provider::{DetectFuture, Provider, UnscopedProvider};

use crate::providers::future::Resolver;
use crate::{ProjectId, Scopes};

pub struct LoadProviderResult<'a, T> {
    pub provider: T,
    pub project_id: ProjectId,
    pub token_future: TryMaybeDone<GetTokenFuture<'a>>,
}

impl<'a, T> LoadProviderResult<'a, T> {
    pub fn map_provider<P>(self, map_fn: impl FnOnce(T) -> P) -> LoadProviderResult<'a, P> {
        LoadProviderResult {
            provider: map_fn(self.provider),
            project_id: self.project_id,
            token_future: self.token_future,
        }
    }
}

/// Base trait for token providers. Actual provider impls
/// are defined by the supertraits [`ScopedTokenProvider`] and [`TokenProvider`].
pub trait BaseTokenProvider: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;
}

pub trait ScopedTokenProvider: BaseTokenProvider {
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_>;

    #[inline]
    fn with_scopes(self, scopes: Scopes) -> ScopedProvider<Self>
    where
        Self: Sized,
    {
        ScopedProvider {
            provider: self,
            scopes,
        }
    }
}

/// For providers that don't need any scopes to generate a token.
pub trait TokenProvider: BaseTokenProvider {
    fn get_token(&self) -> GetTokenFuture<'_>;

    // default impl just clones the inner provider, since all implementors of
    // this trait except ScopedProvider shouldn't care about scopes.
    fn with_new_scope(&self, scopes: Scopes) -> Self
    where
        Self: Sized + Clone,
    {
        let _ = scopes;
        self.clone()
    }
}

impl<B: BaseTokenProvider + ?Sized> BaseTokenProvider for &B {
    #[inline]
    fn name(&self) -> &'static str {
        B::name(self)
    }
}

impl<B: BaseTokenProvider + ?Sized> BaseTokenProvider for std::sync::Arc<B> {
    #[inline]
    fn name(&self) -> &'static str {
        B::name(self)
    }
}

impl<B: BaseTokenProvider + ?Sized> BaseTokenProvider for Box<B> {
    #[inline]
    fn name(&self) -> &'static str {
        B::name(self.as_ref())
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

impl<S> ScopedTokenProvider for Box<S>
where
    S: ScopedTokenProvider,
{
    #[inline]
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        S::get_scoped_token(self, scopes)
    }
}

impl<S> ScopedTokenProvider for std::sync::Arc<S>
where
    S: ScopedTokenProvider,
{
    #[inline]
    fn get_scoped_token(&self, scopes: Scopes) -> GetTokenFuture<'_> {
        S::get_scoped_token(self, scopes)
    }
}

fn _assert_scoped_token_provider_dyn(_: &dyn ScopedTokenProvider) {}

impl<S: TokenProvider + Sized> TokenProvider for &S {
    fn get_token(&self) -> GetTokenFuture<'_> {
        S::get_token(&self)
    }
}

impl<S: TokenProvider + ?Sized> TokenProvider for Box<S> {
    #[inline]
    fn get_token(&self) -> GetTokenFuture<'_> {
        S::get_token(self)
    }
}

impl<S: TokenProvider + ?Sized> TokenProvider for std::sync::Arc<S> {
    #[inline]
    fn get_token(&self) -> GetTokenFuture<'_> {
        S::get_token(self)
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

    #[inline]
    fn with_new_scope(&self, scopes: Scopes) -> Self
    where
        Self: Sized + Clone,
    {
        let mut new = self.clone();
        new.scopes = scopes;
        new
    }
}

fn map_token_future<'a, F, G: Resolver>(
    fut: TryMaybeDone<F>,
    map_fut: impl FnOnce(F) -> GetTokenFuture<'a, G>,
    map_ok: impl FnOnce(F::Ok) -> <GetTokenFuture<'a, G> as TryFuture>::Ok,
) -> TryMaybeDone<GetTokenFuture<'a, G>>
where
    F: TryFuture,
{
    match fut {
        TryMaybeDone::Gone => TryMaybeDone::Gone,
        TryMaybeDone::Done(res) => TryMaybeDone::Done(map_ok(res)),
        TryMaybeDone::Future(fut) => TryMaybeDone::Future(map_fut(fut)),
    }
}

#[cfg(feature = "pinned-token-future")]
pub struct AsyncFnProvider<F> {
    name: &'static str,
    f: F,
}

#[cfg(feature = "pinned-token-future")]
impl<F> AsyncFnProvider<F> {
    pub fn new(name: &'static str, f: F) -> Self {
        Self { name, f }
    }

    pub fn into_auth(self, project_id: ProjectId) -> crate::Auth
    where
        Self: TokenProvider,
        F: 'static,
    {
        crate::Auth::new_from_provider(LoadProviderResult {
            project_id,
            provider: self,
            token_future: TryMaybeDone::Gone,
        })
    }
}

#[cfg(feature = "pinned-token-future")]
impl<F> std::fmt::Debug for AsyncFnProvider<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncFnProvider")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "pinned-token-future")]
impl<F: Send + Sync> BaseTokenProvider for AsyncFnProvider<F> {
    fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(feature = "pinned-token-future")]
impl<F, Err> TokenProvider for AsyncFnProvider<F>
where
    F: AsyncFn() -> Result<crate::Token, Err> + Send + Sync,
    crate::Error: From<Err>,
    for<'a> F::CallRefFuture<'a>: Send + 'static,
{
    fn get_token(&self) -> GetTokenFuture<'_> {
        let future = (self.f)();

        GetTokenFuture::pin(async move { future.await.map_err(crate::Error::from) })
    }
}
