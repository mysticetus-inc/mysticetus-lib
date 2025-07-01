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
#[derive(Clone)]
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
                    tracing::info!(message = "got token from new provider", ?token);
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
        tracing::info!(message = "got token from fallback provider", %expires_at);

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

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

struct AuthState {
    cached: Option<(HeaderValue, Timestamp)>,
    pending_request: PendingRequest,
}

fn build_header(token: &str) -> HeaderValue {
    const BEARER_PREFIX: &str = "Bearer ";
    let mut dst = bytes::BytesMut::with_capacity(BEARER_PREFIX.len() + token.len());

    dst.extend_from_slice(BEARER_PREFIX.as_bytes());
    dst.extend_from_slice(token.as_bytes());

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

        debug_assert_eq!(project_id, id.0.as_ref());

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
                debug_assert_eq!(project_id, proj_id.0.as_ref());
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

        debug_assert_eq!(project_id, proj_id.0.as_ref());

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
        debug_assert_eq!(project_id, proj_id.0.as_ref());
        Ok(Self::new_from_service_account(project_id, svc_acct, scope))
    }

    pub fn get_cached_header(&self) -> Option<HeaderValue> {
        let read_guard = self.state.read().unwrap_or_else(PoisonError::into_inner);

        let (header, expires_at) = read_guard.cached.as_ref()?;

        if is_expired(*expires_at) {
            None
        } else {
            Some(header.clone())
        }
    }

    pub fn get_header(&self) -> GetHeaderResult {
        if let Some(token) = self.get_cached_header() {
            tracing::debug!("returning cached token");
            return GetHeaderResult::Cached(token);
        }

        let mut guard = self.state.write().unwrap_or_else(PoisonError::into_inner);

        // while we were waiting to acquire the write guard, see if
        // another thread already finished polling + updating the token
        match &guard.cached {
            Some((header, expires_at)) if is_expired(*expires_at) => {
                tracing::debug!("returning newly cached token");
                return GetHeaderResult::Cached(header.clone());
            }
            _ => (),
        }

        if !guard.pending_request.is_request_pending() {
            tracing::debug!("starting request for new token");
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

fn is_expired(expires_at: Timestamp) -> bool {
    Timestamp::now() >= expires_at - Duration::from_seconds(15)
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
            match guard.cached {
                Some((ref header, expires_at)) if !is_expired(expires_at) => {
                    tracing::debug!(
                        "got {} byte token, expires at {expires_at}",
                        header.as_bytes().len()
                    );
                    return Poll::Ready(Ok(header.clone()));
                }
                _ => (),
            }

            match std::task::ready!(guard.pending_request.poll(cx)) {
                Ok(Some((header, expires_at))) => {
                    guard.cached = Some((header.clone(), expires_at));
                    return Poll::Ready(Ok(header));
                }
                Ok(None) => unreachable!(),
                Err(err) if *this.retries_left == 0 => return Poll::Ready(Err(err.into())),
                Err(err) => {
                    *this.retries_left -= 1;
                    tracing::warn!(message="error getting auth token", error = ?err, retries_left=*this.retries_left);
                    guard
                        .pending_request
                        .start_request(&this.auth.provider, this.auth.scope);
                }
            }
        }
    }
}
