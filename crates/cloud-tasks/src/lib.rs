use std::borrow::Cow;
use std::fmt;

use gcp_auth_provider::service::AuthSvc;
use gcp_auth_provider::{Auth, Scope};

pub mod error;
mod http;
mod task;

pub use error::Error;
pub use http::HttpRequestBuilder;
pub use task::TaskQueueClient;
use tonic::transport::Channel;

const CLOUD_TASKS_URL: &str = "https://cloudtasks.googleapis.com";
const CLOUD_TASKS_DOMAIN: &str = "cloudtasks.googleapis.com";

pub type Result<T> = ::core::result::Result<T, Error>;

pub struct CloudTaskClient {
    channel: AuthSvc<Channel>,
}

impl CloudTaskClient {
    pub async fn new(scope: Scope) -> Result<Self> {
        let channel = Auth::builder()
            .channel_with_defaults(CLOUD_TASKS_URL, CLOUD_TASKS_DOMAIN)
            .auth(Auth::new_detect().with_scopes(scope))
            .build()
            .await?;

        Ok(Self { channel })
    }

    pub async fn new_from_auth(auth: Auth) -> Result<Self> {
        let channel = Auth::builder()
            .channel_with_defaults(CLOUD_TASKS_URL, CLOUD_TASKS_DOMAIN)
            .auth(auth)
            .build()
            .await?;

        Ok(Self { channel })
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
    channel: Cow<'a, AuthSvc<Channel>>,
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
