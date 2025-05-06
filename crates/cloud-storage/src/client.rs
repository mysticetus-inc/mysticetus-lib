use std::time::Duration;

use gcp_auth_channel::{Auth, AuthChannel};
use tonic::transport::{Channel, ClientTlsConfig};

const STORAGE_DOMAIN: &str = "storage.googleapis.com";
const STORAGE_URL: &str = "https://storage.googleapis.com";
const DEFUALT_KEEPALIVE_DUR: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct StorageClient {
    channel: AuthChannel,
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

async fn build_auth(
    project_id: &'static str,
    scope: gcp_auth_channel::Scope,
) -> crate::Result<Auth> {
    Auth::new(project_id, scope)
        .await
        .map_err(crate::Error::from)
}

impl StorageClient {
    pub async fn new(
        project_id: &'static str,
        scope: gcp_auth_channel::Scope,
    ) -> crate::Result<Self> {
        let (auth, channel) = tokio::try_join!(build_auth(project_id, scope), build_channel())?;

        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

        Ok(Self { channel })
    }

    pub async fn from_auth(auth: Auth) -> crate::Result<Self> {
        let channel = build_channel().await?;

        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

        Ok(Self { channel })
    }

    pub async fn from_auth_future<E>(
        auth_future: impl Future<Output = Result<Auth, E>>,
    ) -> crate::Result<Self>
    where
        E: Into<crate::Error>,
    {
        let (channel, auth) = tokio::try_join!(
            build_channel(),
            
        )?;
        
        let channel = .await?;

        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

        Ok(Self { channel })
    }

    #[inline]
    pub fn auth(&self) -> &Auth {
        self.channel.auth()
    }

    pub async fn from_service_account<P>(
        project_id: &'static str,
        scope: gcp_auth_channel::Scope,
        path: P,
    ) -> crate::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let auth = Auth::new_from_service_account_file(project_id, path.as_ref(), scope)?;
        let channel = build_channel().await?;

        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

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
