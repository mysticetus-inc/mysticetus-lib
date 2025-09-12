use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::future::TryMaybeDone;
use http::HeaderValue;

use crate::{GetTokenFuture, ProjectId, Result, Token, TokenProvider};

pub(crate) mod cache_impl;
pub(crate) mod driver;

pub(crate) use cache_impl::CachedTokenProvider;

#[derive(Debug, Default)]
pub struct TokenState {
    cached: Option<Token>,
    refresher: driver::TokenRefresher,
}

impl TokenState {
    fn new_from_token_fut<P>(token_fut: TryMaybeDone<GetTokenFuture<'static>>, provider: &P) -> Self
    where
        P: TokenProvider + ?Sized,
    {
        match token_fut {
            TryMaybeDone::Done(token) if token.valid_for().is_ok() => Self {
                cached: Some(token),
                refresher: driver::TokenRefresher::new_with_refresh(provider),
            },
            TryMaybeDone::Future(pending_fut) => Self {
                cached: None,
                refresher: driver::TokenRefresher::new_from_future(provider.name(), pending_fut),
            },
            _ => Self {
                cached: None,
                refresher: driver::TokenRefresher::new_with_refresh(provider),
            },
        }
    }

    pub fn get_valid_auth(&self) -> Option<ValidAuth> {
        match self.cached {
            Some(ref token) => match token.valid_for() {
                Ok(valid_for) => Some(ValidAuth {
                    header: token.header().clone(),
                    valid_for,
                }),
                Err(_) => None,
            },
            None => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidAuth {
    pub header: HeaderValue,
    pub valid_for: Duration,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StartNewRequestOnRevoke {
    Yes,
    #[allow(unused)]
    No,
}

/// This trait serves to turn [CachedTokenProvider] into a trait object, to erase the inner generic
/// type.
///
/// The reasoning for this instead of using [CachedTokenProvider<Arc<dyn TokenProvider>>]
/// is that we can have the entire [crate::Auth] type contain a single [Arc], making it fairly
/// compact, and only requiring one reference count to manage.
pub(crate) trait TokenCache: std::fmt::Debug + Send + Sync + 'static {
    fn revoke(&self, start_new_request: StartNewRequestOnRevoke);

    fn project_id(&self) -> ProjectId;

    fn get_cached_header(&self) -> Option<ValidAuth>;

    fn provider_name(&self) -> &'static str;

    fn poll_refresh(&self, cx: &mut Context<'_>) -> Poll<Result<ValidAuth>>;
}

#[derive(Debug)]
pub enum GetHeaderResult<'a> {
    Cached(ValidAuth),
    Refreshing(RefreshHeaderFuture<'a>),
}

impl<'a> GetHeaderResult<'a> {
    pub(super) fn get(inner: &'a Arc<dyn TokenCache>) -> Self {
        match inner.get_cached_header() {
            Some(cached) => Self::Cached(cached),
            None => Self::Refreshing(RefreshHeaderFuture {
                auth: std::borrow::Cow::Borrowed(inner),
            }),
        }
    }

    pub fn into_static(self) -> GetHeaderResult<'static> {
        match self {
            Self::Cached(cache) => GetHeaderResult::Cached(cache),
            Self::Refreshing(refresh) => GetHeaderResult::Refreshing(refresh.into_static()),
        }
    }

    pub async fn into_header(self) -> crate::Result<ValidAuth> {
        match self {
            Self::Cached(cache) => Ok(cache),
            Self::Refreshing(refresh) => refresh.await,
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct RefreshHeaderFuture<'a> {
        auth: std::borrow::Cow<'a, Arc<dyn TokenCache>>,
    }
}

impl<'a> RefreshHeaderFuture<'a> {
    pub fn into_static(self) -> RefreshHeaderFuture<'static> {
        RefreshHeaderFuture {
            auth: std::borrow::Cow::Owned(self.auth.into_owned()),
        }
    }

    pub(crate) fn auth(&self) -> &Arc<dyn TokenCache> {
        &self.auth
    }
}

impl Future for RefreshHeaderFuture<'_> {
    type Output = Result<ValidAuth>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().auth.poll_refresh(cx)
    }
}
