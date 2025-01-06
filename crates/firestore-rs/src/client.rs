//! Inner gRPC channel that handles authentication

use std::path::Path;
use std::time::Duration;

use futures::Future;
use gcp_auth_channel::channel::AuthChannel;
use gcp_auth_channel::{Auth, Scope};
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

async fn build_auth(project_id: &'static str, scope: Scope) -> crate::Result<Auth> {
    let auth = Auth::new(project_id, scope).await?;
    Ok(auth)
}

/// Newtype to the specific type of [`firestore_client::FirestoreClient`] that we'll be using here.
#[derive(Debug, Clone)]
pub struct FirestoreClient {
    channel: AuthChannel,
}

/*
fn wrap_auth_channel(channel: AuthChannel) -> crate::Result<AuthService> {
    channel
        .attach_header()
        .static_key("x-goog-request-params")
        .parse_value("this")?
        .into_intercepted()
}
*/

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
    pub(crate) fn get(&self) -> firestore_client::FirestoreClient<AuthChannel> {
        firestore_client::FirestoreClient::new(self.channel.clone())
    }

    pub fn auth(&self) -> &Auth {
        self.channel.auth()
    }
    pub(crate) fn from_auth_channel(channel: AuthChannel) -> Self {
        Self { channel }
    }

    fn new_inner(auth: Auth, channel: Channel) -> Self {
        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

        Self { channel }
    }

    pub async fn new(project_id: &'static str, scope: Scope) -> crate::Result<Self> {
        // initialize the channel + GCP auth manager concurrently
        let (channel, auth) = tokio::try_join!(build_channel(), build_auth(project_id, scope))?;

        Ok(Self::new_inner(auth, channel))
    }

    pub async fn from_service_account_credentials<P>(
        project_id: &'static str,
        path: P,
        scope: Scope,
    ) -> crate::Result<Self>
    where
        P: AsRef<Path>,
    {
        let auth = Auth::new_from_service_account_file(project_id, path.as_ref(), scope)?;

        let channel = build_channel().await?;

        Ok(Self::new_inner(auth, channel))
    }

    pub async fn from_auth_manager_future<F>(auth_manager_fut: F) -> crate::Result<Self>
    where
        F: Future,
        F::Output: Into<gcp_auth_channel::Auth>,
    {
        // initialize the channel/auth manager
        let (channel, auth_manager) =
            tokio::try_join!(build_channel(), async move { Ok(auth_manager_fut.await) })?;

        Ok(Self::new_inner(auth_manager.into(), channel))
    }

    pub async fn from_auth_manager<A>(auth_manager: A) -> crate::Result<Self>
    where
        A: Into<gcp_auth_channel::Auth>,
    {
        // initialize the channel
        let channel = build_channel().await?;

        Ok(Self::new_inner(auth_manager.into(), channel))
    }

    pub fn project_id(&self) -> &'static str {
        self.channel.auth().project_id()
    }
}
