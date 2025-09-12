//! Inner gRPC channel that handles authentication

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::{Future, TryFutureExt};
use gcp_auth_provider::providers::ScopedTokenProvider;
use gcp_auth_provider::service::AuthSvc;
use gcp_auth_provider::{Auth, Scope};
use net_utils::header::GoogRequestParam;
use protos::firestore::firestore_client;
use tonic::Code;
use tonic::transport::{Channel, ClientTlsConfig};

/// The root firestore destination URL.
const FIRESTORE_DST_URL: &str = match option_env!("FIRESTORE_EMULATOR_HOST") {
    Some(host) => host,
    None => "https://firestore.googleapis.com",
};

/// Firestore domain for TLS config.
const FIRESTORE_DOMAIN: &str = "firestore.googleapis.com";

async fn build_channel() -> crate::Result<Channel> {
    let channel = Channel::from_static(FIRESTORE_DST_URL)
        .tls_config(
            ClientTlsConfig::new()
                .domain_name(FIRESTORE_DOMAIN)
                .with_enabled_roots(),
        )?
        .timeout(Duration::from_secs(60))
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .connect_timeout(Duration::from_secs(5))
        .http2_adaptive_window(true)
        .connect()
        .await?;

    Ok(channel)
}

async fn build_auth(scope: Scope) -> crate::Result<Auth> {
    Auth::new_detect()
        .with_scopes(scope)
        .await
        .map_err(crate::Error::from)
}

/// Newtype to the specific type of [`firestore_client::FirestoreClient`] that we'll be using here.
#[derive(Debug, Clone)]
pub struct FirestoreClient {
    pub(crate) qualified_db_path: Arc<str>,
    pub(crate) channel: AuthSvc<GoogRequestParam<Channel>>,
}

/// Helper trait to convert errors representing 404s as [`Ok(None)`]
pub(crate) trait ResponseExt<R> {
    fn handle_not_found(self) -> Result<Option<R>, tonic::Status>;
}

impl<R> ResponseExt<R> for Result<tonic::Response<R>, tonic::Status> {
    #[inline]
    fn handle_not_found(self) -> Result<Option<R>, tonic::Status> {
        match self {
            Ok(resp) => Ok(Some(resp.into_inner())),
            Err(not_found) if not_found.code() == Code::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl FirestoreClient {
    #[inline]
    pub(crate) fn get(
        &self,
    ) -> firestore_client::FirestoreClient<AuthSvc<GoogRequestParam<Channel>>> {
        firestore_client::FirestoreClient::new(self.channel.clone())
    }

    pub fn auth(&self) -> &Auth {
        self.channel.auth()
    }

    pub(crate) fn new_inner(auth: Auth, channel: Channel, database: &str) -> Self {
        Self::from_auth_svc(auth.into_service(channel), database)
    }

    pub(crate) fn from_auth_svc(svc: AuthSvc<Channel>, database: &str) -> Self {
        let qualified_db_path = Arc::from(format!(
            "projects/{}/databases/{database}",
            svc.auth().project_id().as_str()
        ));

        Self::from_auth_channel(qualified_db_path, svc)
    }

    pub(crate) fn from_auth_channel(
        qualified_db_path: Arc<str>,
        channel: AuthSvc<Channel>,
    ) -> Self {
        let param_bytes = bytes::Bytes::from(format!("database={qualified_db_path}"));
        let param = http::HeaderValue::from_maybe_shared(param_bytes)
            .expect("qualified db path should be a valid header value");

        let channel = channel.map(|channel| GoogRequestParam::new(channel, param));
        Self {
            channel,
            qualified_db_path,
        }
    }

    pub async fn new(scope: Scope, database: &str) -> crate::Result<Self> {
        // initialize the channel + GCP auth manager concurrently
        let (channel, auth) = tokio::try_join!(build_channel(), build_auth(scope))?;

        Ok(Self::new_inner(auth, channel, database))
    }

    pub async fn from_service_account_credentials<P>(
        path: P,
        scope: Scope,
        database: &str,
    ) -> crate::Result<Self>
    where
        P: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        let (svc_account, channel) = tokio::try_join!(
            async move {
                gcp_auth_provider::providers::service_account::ServiceAccount::new_from_path(path)
                    .await
                    .map_err(crate::Error::from)
            },
            build_channel(),
        )?;

        let auth = Auth::new_from_provider(
            svc_account.map_provider(|prov| prov.with_scopes(scope.into())),
        );

        Ok(Self::new_inner(auth, channel, database))
    }

    pub async fn from_auth_manager_future<F>(
        auth_manager_fut: F,
        database: &str,
    ) -> crate::Result<Self>
    where
        F: Future,
        F::Output: Into<gcp_auth_provider::Auth>,
    {
        let infallible_fut = async move {
            Ok(auth_manager_fut.await.into()) as Result<Auth, std::convert::Infallible>
        };

        Self::from_try_auth_manager_future(infallible_fut, database).await
    }

    pub async fn from_try_auth_manager_future<F, Error>(
        auth_manager_fut: F,
        database: &str,
    ) -> crate::Result<Self>
    where
        F: Future<Output = Result<Auth, Error>>,
        crate::Error: From<Error>,
    {
        // initialize the channel/auth manager
        let (channel, auth_manager) = tokio::try_join!(
            build_channel(),
            auth_manager_fut.map_err(crate::Error::from)
        )?;

        Ok(Self::new_inner(auth_manager, channel, database))
    }

    pub async fn from_auth_manager<A>(auth_manager: A, database: &str) -> crate::Result<Self>
    where
        A: Into<gcp_auth_provider::Auth>,
    {
        // initialize the channel
        let channel = build_channel().await?;

        Ok(Self::new_inner(auth_manager.into(), channel, database))
    }

    pub fn project_id(&self) -> &'static str {
        self.channel.auth().project_id().as_str()
    }
}
