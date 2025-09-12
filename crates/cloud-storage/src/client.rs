use std::time::Duration;

use gcp_auth_provider::Auth;
use gcp_auth_provider::service::AuthSvc;
use tonic::transport::{Channel, ClientTlsConfig};

const STORAGE_DOMAIN: &str = "storage.googleapis.com";
const STORAGE_URL: &str = "https://storage.googleapis.com";
const DEFUALT_KEEPALIVE_DUR: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct StorageClient {
    channel: AuthSvc<Channel>,
}

async fn build_channel() -> crate::Result<Channel> {
    Channel::from_static(STORAGE_URL)
        .user_agent("cloud-storage-rs")?
        .tcp_keepalive(Some(DEFUALT_KEEPALIVE_DUR))
        .tls_config(
            ClientTlsConfig::new()
                .domain_name(STORAGE_DOMAIN)
                .with_enabled_roots(),
        )?
        .connect()
        .await
        .map_err(crate::Error::from)
}

async fn build_auth(scopes: gcp_auth_provider::Scopes) -> crate::Result<Auth> {
    Auth::new_detect()
        .with_scopes(scopes)
        .await
        .map_err(crate::Error::from)
}

impl StorageClient {
    pub async fn new(scopes: gcp_auth_provider::Scopes) -> crate::Result<Self> {
        let (auth, channel) = tokio::try_join!(build_auth(scopes), build_channel())?;

        let channel = auth.into_service(channel);

        Ok(Self { channel })
    }

    pub async fn from_auth(auth: Auth) -> crate::Result<Self> {
        let channel = build_channel().await?;

        let channel = auth.into_service(channel);

        Ok(Self { channel })
    }

    pub async fn from_auth_future<E>(
        auth_future: impl Future<Output = Result<Auth, E>>,
    ) -> crate::Result<Self>
    where
        E: Into<crate::Error>,
    {
        let (channel, auth) = tokio::try_join!(build_channel(), async move {
            auth_future.await.map_err(Into::into)
        },)?;

        let channel = auth.into_service(channel);

        Ok(Self { channel })
    }

    #[inline]
    pub fn auth(&self) -> &Auth {
        self.channel.auth()
    }

    pub async fn from_service_account(
        scopes: gcp_auth_provider::Scopes,
        path: impl Into<std::path::PathBuf>,
    ) -> crate::Result<Self> {
        let (auth, channel) = tokio::try_join!(
            async move {
                Auth::from_service_account_file(path.into(), scopes)
                    .await
                    .map_err(crate::Error::from)
            },
            build_channel()
        )?;

        let channel = auth.into_service(channel);

        Ok(Self { channel })
    }

    pub fn bucket<B>(&self, bucket: B) -> crate::bucket::BucketClient
    where
        B: AsRef<str>,
    {
        crate::bucket::BucketClient::new(self.channel.clone(), bucket.as_ref())
    }

    pub fn into_bucket<B>(self, bucket: B) -> crate::bucket::BucketClient
    where
        B: AsRef<str>,
    {
        crate::bucket::BucketClient::new(self.channel, bucket.as_ref())
    }
}
