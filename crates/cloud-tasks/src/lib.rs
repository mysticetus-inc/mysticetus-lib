use std::borrow::Cow;
use std::fmt;

use gcp_auth_channel::{Auth, AuthChannel, Scope};

pub mod error;
mod http;
mod task;

pub use error::Error;
pub use http::HttpRequestBuilder;
pub use task::TaskQueueClient;
use tonic::transport::{Channel, ClientTlsConfig};

const CLOUD_TASKS_URL: &str = "https://cloudtasks.googleapis.com";
const CLOUD_TASKS_DOMAIN: &str = "cloudtasks.googleapis.com";

pub type Result<T> = ::core::result::Result<T, Error>;

async fn build_channel() -> crate::Result<Channel> {
    let channel = Channel::from_static(CLOUD_TASKS_URL)
        .tls_config(
            ClientTlsConfig::new()
                .domain_name(CLOUD_TASKS_DOMAIN)
                .with_enabled_roots(),
        )?
        .timeout(std::time::Duration::from_secs(60))
        .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
        .connect_timeout(std::time::Duration::from_secs(5))
        .http2_adaptive_window(true)
        .connect()
        .await?;

    Ok(channel)
}

pub struct CloudTaskClient {
    channel: AuthChannel,
}

impl CloudTaskClient {
    pub async fn new(project_id: &'static str, scope: Scope) -> Result<Self> {
        let (channel, auth) = tokio::try_join!(build_channel(), async move {
            gcp_auth_channel::Auth::new(project_id, scope)
                .await
                .map_err(crate::Error::Auth)
        })?;

        Ok(Self {
            channel: AuthChannel::builder()
                .with_auth(auth)
                .with_channel(channel)
                .build(),
        })
    }

    pub async fn new_from_auth(auth: Auth) -> Result<Self> {
        let channel = build_channel().await?;
        Ok(Self {
            channel: AuthChannel::builder()
                .with_auth(auth)
                .with_channel(channel)
                .build(),
        })
    }

    pub fn task_client(&self) -> TaskClientBuilder<'_, ()> {
        TaskClientBuilder {
            channel: Cow::Borrowed(&self.channel),
            location_id: (),
        }
    }

    pub fn into_task_client(self) -> TaskClientBuilder<'static, ()> {
        TaskClientBuilder {
            channel: Cow::Owned(self.channel),
            location_id: (),
        }
    }
}

pub struct TaskClientBuilder<'a, LocationId> {
    channel: Cow<'a, AuthChannel>,
    location_id: LocationId,
}

impl<'a> TaskClientBuilder<'a, ()> {
    pub fn location<L>(self, location_id: L) -> TaskClientBuilder<'a, L> {
        TaskClientBuilder {
            channel: self.channel,
            location_id,
        }
    }
}

impl<L> TaskClientBuilder<'_, L>
where
    L: fmt::Display,
{
    pub fn queue<Q: fmt::Display>(self, queue: Q) -> TaskQueueClient {
        let queue = format!(
            "projects/{project_id}/locations/{location}/queues/{queue}",
            project_id = self.channel.auth().project_id(),
            location = self.location_id,
            queue = queue,
        );

        TaskQueueClient::new(self.channel.into_owned(), queue.into_boxed_str())
    }
}
