//! Transport-independant authentication via [`Auth`]
use std::fmt;
use std::future::{Future, IntoFuture};
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, PoisonError, RwLock};
use std::task::{Context, Poll};

use gcp_auth::Token;
use http::HeaderValue;

#[cfg(any(debug_assertions, feature = "local-gcloud"))]
mod local_gcloud_provider;

mod pending;
use pending::PendingRequest;

use crate::Scope;

/// Struct encapsulating all state + the auth manager itself.
#[derive(Clone)]
pub struct Auth {
    provider: Arc<dyn gcp_auth::TokenProvider>,
    scope: Scope,
    project_id: &'static str,
    state: Arc<RwLock<AuthState>>,
}

#[cfg(feature = "emulator")]
pub struct EmulatorTokenProvider {
    project_id: &'static str,
    fake_token: Arc<Token>,
}

#[cfg(feature = "emulator")]
#[tonic::async_trait]
impl gcp_auth::TokenProvider for EmulatorTokenProvider {
    async fn token(&self, _scopes: &[&str]) -> Result<Arc<Token>, gcp_auth::Error> {
        Ok(Arc::clone(&self.fake_token))
    }

    async fn project_id(&self) -> Result<Arc<str>, gcp_auth::Error> {
        Ok(Arc::from(self.project_id))
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

struct AuthState {
    cached_header: Option<(Arc<Token>, HeaderValue)>,
    pending_request: PendingRequest,
}

fn build_header(token: &Token) -> HeaderValue {
    const BEARER_PREFIX: &str = "Bearer ";
    let access_token = token.as_str();

    let mut dst = bytes::BytesMut::with_capacity(BEARER_PREFIX.len() + access_token.len());

    dst.extend_from_slice(BEARER_PREFIX.as_bytes());
    dst.extend_from_slice(access_token.as_bytes());

    // SAFETY: we only append bytes from utf-8 strings to 'dst', therefore
    // this is safe and we can skip the checks.
    unsafe { HeaderValue::from_maybe_shared_unchecked(dst.freeze()) }
}

pub enum GetHeaderResult {
    Cached(HeaderValue),
    Refreshing(RefreshHeaderFuture),
}

impl IntoFuture for GetHeaderResult {
    type Output = crate::Result<HeaderValue>;
    type IntoFuture = GetHeaderFuture;

    fn into_future(self) -> Self::IntoFuture {
        match self {
            Self::Cached(header) => GetHeaderFuture::Cached {
                header: Some(header),
            },
            Self::Refreshing(future) => GetHeaderFuture::Refreshing { future },
        }
    }
}

pin_project_lite::pin_project! {
    #[project = GetHeaderFutureProj]
    pub enum GetHeaderFuture {
        Cached { header: Option<HeaderValue> },
        Refreshing {
            #[pin]
            future: RefreshHeaderFuture,
        },
    }
}

impl Future for GetHeaderFuture {
    type Output = crate::Result<HeaderValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        match this {
            GetHeaderFutureProj::Cached { header } => match header.take() {
                Some(header) => Poll::Ready(Ok(header)),
                None => panic!("GetHeaderFuture polled after completion"),
            },
            GetHeaderFutureProj::Refreshing { future } => {
                let result = std::task::ready!(future.poll(cx));
                // set to None, so we panic with a useful message if polled again
                self.as_mut().set(GetHeaderFuture::Cached { header: None });
                Poll::Ready(result)
            }
        }
    }
}

impl Auth {
    pub const fn project_id(&self) -> &'static str {
        self.project_id
    }

    pub fn inner_auth(&self) -> &Arc<dyn gcp_auth::TokenProvider> {
        &self.provider
    }

    pub fn with_new_scope(&self, scope: Scope) -> Self {
        if scope == self.scope {
            return self.clone();
        }

        Self {
            provider: Arc::clone(&self.provider),
            scope,
            project_id: self.project_id,
            state: Arc::new(RwLock::new(AuthState {
                cached_header: None,
                pending_request: PendingRequest::new(Arc::clone(&self.provider), scope),
            })),
        }
    }

    pub async fn new_cloud_platform(project_id: &'static str) -> crate::Result<Self> {
        Self::new(project_id, Scope::CloudPlatformAdmin).await
    }

    #[cfg(feature = "emulator")]
    pub fn new_emulator(project_id: &'static str, scope: Scope) -> Self {
        const FAKE_TOKEN_JSON: &str =
            "{\"access_token\":\"notarealtoken\",\"expires_in\":100000000}";

        let token = serde_json::from_str(FAKE_TOKEN_JSON).expect("is valid");

        let provider: Arc<dyn gcp_auth::TokenProvider> = Arc::new(EmulatorTokenProvider {
            project_id,
            fake_token: Arc::new(token),
        });

        Self {
            project_id,
            scope,
            provider: Arc::clone(&provider),
            state: Arc::new(RwLock::new(AuthState {
                cached_header: None,
                pending_request: PendingRequest::new(provider, scope),
            })),
        }
    }

    /// Gets an [`Auth`] instance that only uses a local `gcloud` cli installation.
    /// Panics if `gcloud` was not found, or if there was an error trying to find it (via `which`).
    ///
    /// **Should only be used locally**
    #[cfg(any(debug_assertions, feature = "local-gcloud"))]
    pub fn new_gcloud(project_id: &'static str, scope: Scope) -> Self {
        match local_gcloud_provider::LocalGCloudProvider::try_load() {
            Ok(Some(provider)) => Self::new_from_provider(project_id, Arc::from(provider), scope),
            Ok(None) => panic!("gcloud cli not found"),
            Err(error) => panic!("error trying to find gcloud: {error}"),
        }
    }

    // the gcp_auth gcloud provider is broken, so we try to use our own in tests
    #[cfg(any(debug_assertions, feature = "local-gcloud"))]
    pub async fn new(project_id: &'static str, scope: Scope) -> crate::Result<Self> {
        println!("looking for local gcloud provider");
        let provider = match local_gcloud_provider::LocalGCloudProvider::try_load() {
            Ok(Some(provider)) => {
                println!("using local gcloud provider");
                Arc::from(provider)
            }
            Ok(None) => {
                println!("gcloud not found, attempting to use fallback");
                gcp_auth::provider().await?
            }
            Err(error) => {
                println!("failed to load local gcloud, attempting to use fallback: {error}");
                gcp_auth::provider().await?
            }
        };
        Ok(Self::new_from_provider(project_id, provider, scope))
    }

    #[cfg(not(any(debug_assertions, feature = "local-gcloud")))]
    pub async fn new(project_id: &'static str, scope: Scope) -> crate::Result<Self> {
        let provider = gcp_auth::provider().await?;
        Ok(Self::new_from_provider(project_id, provider, scope))
    }

    pub fn new_from_provider(
        project_id: &'static str,
        provider: Arc<dyn gcp_auth::TokenProvider>,
        scope: Scope,
    ) -> Self {
        let provider_clone = Arc::clone(&provider);
        Self {
            provider,
            scope,
            project_id,
            state: Arc::new(RwLock::new(AuthState {
                cached_header: None,
                pending_request: PendingRequest::new(provider_clone, scope),
            })),
        }
    }

    fn new_from_service_account(
        project_id: &'static str,
        service_account: gcp_auth::CustomServiceAccount,
        scope: Scope,
    ) -> Self {
        #[cfg(debug_assertions)]
        if let Some(proj_id) = service_account.project_id() {
            debug_assert_eq!(project_id, proj_id);
        }

        Self::new_from_provider(project_id, Arc::from(service_account), scope)
    }

    pub fn new_from_service_account_json(
        project_id: &'static str,
        json: &str,
        scope: Scope,
    ) -> crate::Result<Self> {
        let provider = gcp_auth::CustomServiceAccount::from_json(json)?;
        Ok(Self::new_from_service_account(project_id, provider, scope))
    }

    pub fn new_from_service_account_file(
        project_id: &'static str,
        path: impl AsRef<Path>,
        scope: Scope,
    ) -> crate::Result<Self> {
        let provider = gcp_auth::CustomServiceAccount::from_file(path.as_ref())?;
        Ok(Self::new_from_service_account(project_id, provider, scope))
    }

    pub fn get_cached_header(&self) -> Option<HeaderValue> {
        let read_guard = self.state.read().unwrap_or_else(PoisonError::into_inner);

        let (token, header) = read_guard.cached_header.as_ref()?;

        if token.has_expired() {
            None
        } else {
            Some(header.clone())
        }
    }

    pub fn get_header(&self) -> GetHeaderResult {
        if let Some(token) = self.get_cached_header() {
            return GetHeaderResult::Cached(token);
        }

        let mut guard = self.state.write().unwrap_or_else(PoisonError::into_inner);

        // while we were waiting to acquire the write guard, see if
        // another thread already finished polling + updating the token
        match &guard.cached_header {
            Some((token, header)) if !token.has_expired() => {
                return GetHeaderResult::Cached(header.clone());
            }
            _ => (),
        }

        if !guard.pending_request.is_request_pending() {
            guard
                .pending_request
                .start_request(Arc::clone(&self.provider), self.scope);
        }

        drop(guard);

        GetHeaderResult::Refreshing(RefreshHeaderFuture {
            auth: self.clone(),
            retries_left: 5,
        })
    }
}

pin_project_lite::pin_project! {
    pub struct RefreshHeaderFuture {
        auth: Auth,
        retries_left: usize,
    }
}

impl Future for RefreshHeaderFuture {
    type Output = crate::Result<HeaderValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut guard = this
            .auth
            .state
            .write()
            .unwrap_or_else(PoisonError::into_inner);

        loop {
            match guard.cached_header {
                Some((ref token, ref header)) if !token.has_expired() => {
                    return Poll::Ready(Ok(header.clone()));
                }
                _ => (),
            }

            match std::task::ready!(guard.pending_request.poll(cx)) {
                Ok(Some(token)) => {
                    let header = build_header(&token);
                    guard.cached_header = Some((token, header.clone()));
                    return Poll::Ready(Ok(header));
                }
                Ok(None) => unreachable!(),
                Err(err) if *this.retries_left == 0 => return Poll::Ready(Err(err.into())),
                Err(err) => {
                    *this.retries_left -= 1;
                    tracing::warn!(message="error getting auth token", error = ?err, retries_left=*this.retries_left);
                    guard
                        .pending_request
                        .start_request(this.auth.provider.clone(), this.auth.scope);
                }
            }
        }
    }
}
