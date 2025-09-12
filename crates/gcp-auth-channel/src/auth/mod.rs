//! Transport-independant authentication via [`Auth`]
use std::fmt;
use std::future::{Future, IntoFuture};
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, PoisonError, RwLock};
use std::task::{Context, Poll};

use http::HeaderValue;

#[cfg(any(debug_assertions, feature = "local-gcloud"))]
mod local_gcloud_provider;

mod pending;
use pending::PendingRequest;
use timestamp::{Duration, Timestamp};

use crate::Scope;

/// Struct encapsulating all state + the auth manager itself.
#[derive(Debug, Clone)]
pub struct Auth {
    provider: Arc<Provider>,
    scope: Scope,
    project_id: &'static str,
    state: Arc<RwLock<AuthState>>,
}

pub struct Provider {
    new: Option<Arc<gcp_auth_provider::TokenProvider>>,
    use_fallback: AtomicBool,
    fallback: tokio::sync::OnceCell<Arc<dyn gcp_auth::TokenProvider>>,
}

impl std::fmt::Debug for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Provider")
            .field("new", &self.new.as_ref())
            .field("use_fallback", &self.use_fallback)
            .finish_non_exhaustive()
    }
}

impl Provider {
    fn get_token(
        self: &Arc<Self>,
        scope: Scope,
    ) -> impl Future<Output = crate::Result<(HeaderValue, Timestamp)>> + Send + 'static {
        let this = Arc::clone(self);
        async move {
            if this.use_fallback.load(std::sync::atomic::Ordering::Relaxed) {
                return this.get_from_fallback(scope).await;
            }

            let Some(ref new) = this.new else {
                this.use_fallback
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                return this.get_from_fallback(scope).await;
            };

            let result = new
                .get_token(match scope {
                    Scope::CloudPlatformAdmin => gcp_auth_provider::Scopes::CLOUD_PLATFORM_ADMIN,
                    Scope::CloudPlatformReadOnly => {
                        gcp_auth_provider::Scopes::CLOUD_PLATFORM_READ_ONLY
                    }
                    Scope::BigQueryAdmin => gcp_auth_provider::Scopes::BIG_QUERY_ADMIN,
                    Scope::BigQueryReadWrite => gcp_auth_provider::Scopes::BIG_QUERY_READ_WRITE,
                    Scope::BigQueryReadOnly => gcp_auth_provider::Scopes::BIG_QUERY_READ_ONLY,
                    Scope::Firestore => gcp_auth_provider::Scopes::FIRESTORE,
                    Scope::GcsAdmin => gcp_auth_provider::Scopes::GCS_ADMIN,
                    Scope::GcsReadWrite => gcp_auth_provider::Scopes::GCS_READ_WRITE,
                    Scope::GcsReadOnly => gcp_auth_provider::Scopes::GCS_READ_ONLY,
                    Scope::CloudTasks => gcp_auth_provider::Scopes::CLOUD_TASKS,
                    Scope::PubSub => gcp_auth_provider::Scopes::PUB_SUB,
                    Scope::SpannerAdmin => gcp_auth_provider::Scopes::SPANNER_ADMIN,
                    Scope::SpannerData => gcp_auth_provider::Scopes::SPANNER_DATA,
                    Scope::FirestoreRealtimeDatabase => {
                        gcp_auth_provider::Scopes::REALTIME_DATABASE
                    }
                })
                .await;

            match result {
                Ok(token) => {
                    tracing::info!(message = "got token from new provider", ?token, provider = ?this);
                    Ok((build_header(token.access_token()), token.expires_at()))
                }
                Err(error) => {
                    tracing::error!(message = "new auth provider error, switching to fallback", ?error, provider = ?this.new);
                    this.use_fallback
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                    this.get_from_fallback(scope).await
                }
            }
        }
    }

    async fn get_from_fallback(&self, scope: Scope) -> crate::Result<(HeaderValue, Timestamp)> {
        let provider = self.fallback.get_or_try_init(gcp_auth::provider).await?;

        let token = provider.token(&[scope.scope_uri()]).await?;

        let expires_at = token.expires_at().into();
        tracing::info!(message = "got token from fallback provider", %expires_at, provider = ?self);

        Ok((build_header(token.as_str()), expires_at))
    }
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

struct AuthState {
    cached: Option<(HeaderValue, Timestamp)>,
    pending_request: PendingRequest,
}

impl fmt::Debug for AuthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthState")
            .field("cached", &self.cached)
            .field(
                "has_pending_request",
                &self.pending_request.is_request_pending(),
            )
            .finish()
    }
}

fn build_header(token: &str) -> HeaderValue {
    const BEARER_PREFIX: &str = "Bearer ";
    let mut dst = bytes::BytesMut::with_capacity(BEARER_PREFIX.len() + token.len());

    dst.extend_from_slice(BEARER_PREFIX.as_bytes());
    dst.extend_from_slice(token.as_bytes());

    // SAFETY: we only append bytes from utf-8 strings to 'dst', therefore
    // this is safe and we can skip the checks.
    let mut header = unsafe { HeaderValue::from_maybe_shared_unchecked(dst.freeze()) };
    header.set_sensitive(true);
    header
}

#[derive(Debug)]
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

    pub fn with_new_scope(&self, scope: Scope) -> Self {
        if scope == self.scope {
            return self.clone();
        }

        Self {
            provider: Arc::clone(&self.provider),
            scope,
            project_id: self.project_id,
            state: Arc::new(RwLock::new(AuthState {
                cached: None,
                pending_request: PendingRequest::new(&self.provider, scope),
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
            Ok(Some(provider)) => Self::new_from_provider(
                project_id,
                Arc::new(Provider {
                    new: None,
                    use_fallback: AtomicBool::new(true),
                    fallback: tokio::sync::OnceCell::new_with(Some(Arc::from(provider))),
                }),
                scope,
            ),
            Ok(None) => panic!("gcloud cli not found"),
            Err(error) => panic!("error trying to find gcloud: {error}"),
        }
    }

    pub async fn new(project_id: &'static str, scope: Scope) -> crate::Result<Self> {
        let (inner_provider, id) = gcp_auth_provider::TokenProvider::detect().await?;

        debug_assert_eq!(project_id, id.as_ref());

        let provider = Arc::new(Provider {
            new: Some(Arc::new(inner_provider)),
            use_fallback: AtomicBool::new(false),
            fallback: tokio::sync::OnceCell::const_new(),
        });

        Ok(Self::new_from_provider(project_id, provider, scope))
    }

    pub async fn new_metadata_server(
        project_id: &'static str,
        scope: Scope,
    ) -> crate::Result<Self> {
        match gcp_auth_provider::metadata::MetadataServer::new().await {
            Ok((provider, proj_id)) => {
                debug_assert_eq!(project_id, proj_id.as_ref());
                Ok(Self::new_from_provider(
                    project_id,
                    Arc::new(Provider {
                        new: Some(Arc::new(gcp_auth_provider::TokenProvider::MetadataServer(
                            provider,
                        ))),
                        use_fallback: AtomicBool::new(false),
                        fallback: tokio::sync::OnceCell::const_new(),
                    }),
                    scope,
                ))
            }
            Err(error) => {
                tracing::warn!(
                    message = "failed to connect to the metadata instance, falling back",
                    ?error
                );
                Self::new(project_id, scope).await
            }
        }
    }

    pub fn new_from_provider(
        project_id: &'static str,
        provider: Arc<Provider>,
        scope: Scope,
    ) -> Self {
        let pending_request = PendingRequest::new(&provider, scope);
        Self {
            provider,
            scope,
            project_id,
            state: Arc::new(RwLock::new(AuthState {
                cached: None,
                pending_request,
            })),
        }
    }

    fn new_from_service_account(
        project_id: &'static str,
        service_account: gcp_auth_provider::service_account::ServiceAccount,
        scope: Scope,
    ) -> Self {
        let provider = Provider {
            fallback: tokio::sync::OnceCell::new(),
            use_fallback: AtomicBool::new(false),
            new: Some(Arc::new(gcp_auth_provider::TokenProvider::ServiceAccount(
                service_account,
            ))),
        };

        Self::new_from_provider(project_id, Arc::new(provider), scope)
    }

    pub fn new_from_service_account_json(
        project_id: &'static str,
        json: &str,
        scope: Scope,
    ) -> crate::Result<Self> {
        let (svc_acct, proj_id) =
            gcp_auth_provider::service_account::ServiceAccount::new_from_json_bytes(
                json.as_bytes(),
            )?;

        debug_assert_eq!(project_id, proj_id.as_ref());

        Ok(Self::new_from_service_account(project_id, svc_acct, scope))
    }

    pub async fn new_from_service_account_file(
        project_id: &'static str,
        path: impl AsRef<Path>,
        scope: Scope,
    ) -> crate::Result<Self> {
        let (svc_acct, proj_id) =
            gcp_auth_provider::service_account::ServiceAccount::new_from_json_file(path.as_ref())
                .await?;
        debug_assert_eq!(project_id, proj_id.as_ref());
        Ok(Self::new_from_service_account(project_id, svc_acct, scope))
    }

    pub fn get_cached_header(&self) -> Option<(HeaderValue, Duration)> {
        let read_guard = self.state.read().unwrap_or_else(PoisonError::into_inner);

        let (header, expires_at) = read_guard.cached.as_ref()?;

        match TokenStatus::new(*expires_at) {
            TokenStatus::Expired => None,
            TokenStatus::Valid { valid_for } => Some((header.clone(), valid_for)),
        }
    }

    pub fn start_token_request(&self, force_refresh: bool) -> bool {
        let mut guard = self.state.write().unwrap_or_else(PoisonError::into_inner);

        if !force_refresh {
            if let Some((_, expires_at)) = guard.cached {
                if !TokenStatus::new(expires_at).is_expired() {
                    return false;
                }
            }
        }

        if guard.pending_request.is_request_pending() {
            return false;
        }

        guard
            .pending_request
            .start_request(&self.provider, self.scope);

        true
    }

    pub fn revoke_token(&self, start_new_request: bool) {
        let mut guard = self.state.write().unwrap_or_else(PoisonError::into_inner);

        guard.cached = None;

        if start_new_request && !guard.pending_request.is_request_pending() {
            guard
                .pending_request
                .start_request(&self.provider, self.scope);
        }
    }

    pub fn get_header(&self) -> GetHeaderResult {
        if let Some((token, valid_for)) = self.get_cached_header() {
            tracing::debug!(message = "returning cached token", auth = ?self, %valid_for, project_id = self.project_id);
            return GetHeaderResult::Cached(token);
        }

        let mut guard = self.state.write().unwrap_or_else(PoisonError::into_inner);

        // while we were waiting to acquire the write guard, see if
        // another thread already finished polling + updating the token
        match &guard.cached {
            Some((header, expires_at)) => match TokenStatus::new(*expires_at) {
                TokenStatus::Valid { valid_for } => {
                    tracing::debug!(message = "returning newly cached token", auth = ?self, %valid_for, project_id = self.project_id);
                    return GetHeaderResult::Cached(header.clone());
                }
                TokenStatus::Expired => guard.cached = None,
            },
            _ => (),
        }

        if !guard.pending_request.is_request_pending() {
            tracing::debug!(message = "starting request for new token", auth = ?self, project_id = self.project_id);
            guard
                .pending_request
                .start_request(&self.provider, self.scope);
        }

        drop(guard);

        GetHeaderResult::Refreshing(RefreshHeaderFuture {
            auth: self.clone(),
            retries_left: 5,
        })
    }
}

#[derive(Debug)]
enum TokenStatus {
    Expired,
    Valid { valid_for: Duration },
}

impl TokenStatus {
    pub fn is_expired(&self) -> bool {
        matches!(self, Self::Expired)
    }

    pub fn new(expires_at: Timestamp) -> Self {
        let now = Timestamp::now();

        let expires_at_with_buffer = expires_at - Duration::from_seconds(30);

        if expires_at_with_buffer <= now {
            Self::Expired
        } else {
            let valid_for = expires_at_with_buffer - now;
            Self::Valid { valid_for }
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
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
            match guard.cached {
                Some((ref header, expires_at)) => match TokenStatus::new(expires_at) {
                    TokenStatus::Valid { valid_for } => {
                        tracing::debug!(
                            "got {} byte token, valid for {valid_for}",
                            header.as_bytes().len()
                        );
                        return Poll::Ready(Ok(header.clone()));
                    }
                    _ => (),
                },
                _ => (),
            }

            match std::task::ready!(guard.pending_request.poll(cx)) {
                Ok(Some((header, expires_at))) => {
                    guard.cached = Some((header.clone(), expires_at));
                    return Poll::Ready(Ok(header));
                }
                Ok(None) => guard
                    .pending_request
                    .start_request(&this.auth.provider, this.auth.scope),
                Err(err) if *this.retries_left == 0 => return Poll::Ready(Err(err.into())),
                Err(err) => {
                    *this.retries_left -= 1;
                    tracing::warn!(message="error getting auth token", auth = ?this.auth, error = ?err, retries_left=*this.retries_left);
                    guard
                        .pending_request
                        .start_request(&this.auth.provider, this.auth.scope);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_expired_checks() {
        let now = Timestamp::now();

        let now_plus_1_hr = now + Duration::from_seconds(3600);

        assert!(TokenStatus::new(now + Duration::from_seconds(20)).is_expired());
        assert!(!TokenStatus::new(now_plus_1_hr).is_expired());
    }

    #[tokio::test]
    async fn test_long_running() -> Result<(), crate::Error> {
        tracing_subscriber::fmt().init();

        let (svc, proj_id) = gcp_auth_provider::service_account::ServiceAccount::new_from_env()
            .await?
            .expect("GOOGLE_APPLICATION_CREDENTIALS not set");

        assert_eq!(proj_id.as_ref(), "mysticetus-oncloud");

        let provider = Arc::new(Provider {
            new: Some(Arc::new(gcp_auth_provider::TokenProvider::ServiceAccount(
                svc,
            ))),
            use_fallback: AtomicBool::new(false),
            fallback: tokio::sync::OnceCell::new(),
        });

        let auth =
            Auth::new_from_provider("mysticetus-oncloud", provider, Scope::CloudPlatformReadOnly);

        let header = auth.get_header().await?;
        tracing::info!("initial header: {header:#?}");

        let start = Instant::now();

        let mut new_count = 0;

        loop {
            tokio::time::sleep(Duration::from_seconds(5 * 60).into()).await;
            let elapsed = Duration::from(start.elapsed());
            tracing::info!("elapsed: {elapsed:?}");

            match dbg!(auth.get_header()) {
                GetHeaderResult::Cached(cached) => tracing::info!("got cached token: {cached:?}"),
                GetHeaderResult::Refreshing(pending) => {
                    let new = pending.await?;
                    new_count += 1;
                    tracing::info!("got new token ({new_count}): {new:?}");
                }
            }

            if Duration::from(elapsed) >= Duration::from_seconds(3660) {
                assert_ne!(0, new_count, "we should have gotten at least 1 new token");
                return Ok(());
            }
        }
    }
}
