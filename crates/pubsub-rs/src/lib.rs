#![feature(maybe_uninit_slice)]
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::{Future, Stream};
use gcp_auth_provider::service::AuthSvc;
use gcp_auth_provider::{Auth, Scopes};
use protos::pubsub::{self, Topic, publisher_client};
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;
use tonic::transport::Channel;

#[macro_use]
extern crate tracing;

pub mod error;
pub mod publisher;
#[cfg(feature = "subscriber")]
pub mod subscriber;
pub mod topic;
mod util;
pub use error::Error;
pub use topic::TopicClient;

const PUBSUB_URL: &str = "https://pubsub.googleapis.com";

const PUBSUB_DOMAIN: &str = "pubsub.googleapis.com";

#[derive(Debug, Clone)]
pub struct PubSubClient {
    pub(crate) channel: AuthSvc<Channel>,
}

fn channel_config() -> gcp_auth_provider::channel::ChannelOptions {
    let mut cfg = gcp_auth_provider::channel::ChannelOptions::new(PUBSUB_URL);
    cfg.domain(PUBSUB_DOMAIN).default_tls();
    cfg
}

impl PubSubClient {
    pub async fn new() -> Result<Self, Error> {
        let channel = Auth::builder()
            .channel(channel_config())
            .auth(Auth::new_detect())
            .build()
            .await?;

        debug!(
            message = "initialized PubSubClient",
            project_id = %channel.auth().project_id()
        );

        Ok(Self { channel })
    }

    pub async fn from_service_account(
        path: impl Into<PathBuf>,
        scopes: impl Into<Scopes>,
    ) -> Result<Self, Error> {
        let channel = Auth::builder()
            .channel(channel_config())
            .auth(Auth::from_service_account_file(path.into(), scopes.into()))
            .build()
            .await?;

        debug!(
            message = "initialized PubSubClient",
            project_id = %channel.auth().project_id(),
        );

        Ok(Self { channel })
    }

    pub async fn from_auth(auth: Auth) -> Result<Self, Error> {
        let channel = Auth::builder()
            .auth(auth)
            .channel(channel_config())
            .build()
            .await?;

        debug!(
            message = "initialized PubSubClient",
            project_id = %channel.auth().project_id(),
        );

        Ok(Self { channel })
    }

    pub async fn create_topic<T>(&self, topic: T) -> Result<TopicClient, Error>
    where
        T: AsRef<str>,
    {
        let req_topic =
            util::make_default_topic(self.channel.auth().project_id().as_str(), topic.as_ref());

        debug!(
            message = "creating topic...",
            topic_name = req_topic.name.as_str()
        );

        // clone the client here, that way we can use it for the initail request and avoid
        // the need to make &self -> &mut self
        let mut channel = self.channel.clone();
        let topic = publisher_client::PublisherClient::new(&mut channel)
            .create_topic(req_topic)
            .await?
            .into_inner();

        debug!(
            message = "successfully created topic",
            topic_name = topic.name.as_str()
        );

        Ok(topic::TopicClient::new_from_topic(topic, channel))
    }

    /// Builds a [`TopicClient`] with a known topic name. Does no checking to make sure the topic
    /// exists, or if the fields of the [`Topic`] item are valid/correct.
    ///
    /// To get a known topic with the correct fields, use [`PubSubClient::get_topic`].
    pub fn topic<S>(&self, topic: S) -> TopicClient
    where
        S: AsRef<str>,
    {
        TopicClient::new_from_name(topic.as_ref(), self.channel.clone())
    }

    #[cfg(feature = "subscriber")]
    pub async fn get_subscriber(
        &self,
        subscription: impl AsRef<str>,
    ) -> Result<subscriber::Subscriber, Error> {
        let subscription = util::make_qualified_subscription_name(
            self.channel.auth().project_id(),
            subscription.as_ref(),
        );

        let mut chan = self.channel.clone().with_scope(Scope::PubSub);

        let req = pubsub::GetSubscriptionRequest { subscription };

        let resp = pubsub::subscriber_client::SubscriberClient::new(&mut chan)
            .get_subscription(req)
            .await?;

        Ok(subscriber::Subscriber::new(chan, resp.into_inner()))
    }

    pub async fn get_topic<S>(&self, topic: S) -> Result<TopicClient, Error>
    where
        S: AsRef<str>,
    {
        let topic = util::make_qualified_topic_name(
            self.channel.auth().project_id().as_str(),
            topic.as_ref(),
        );

        debug!(message = "retrieving topic...", topic = topic.as_str());

        let mut channel = self.channel.clone();

        let topic = publisher_client::PublisherClient::new(&mut channel)
            .get_topic(pubsub::GetTopicRequest { topic })
            .await?
            .into_inner();

        debug!(
            message = "successfully retrieved topic",
            topic_name = topic.name.as_str()
        );

        Ok(TopicClient::new_from_topic(topic, channel))
    }

    #[must_use = "'TopicList' is a stream that must be polled"]
    pub fn list_topics(&self) -> TopicList {
        let project_id = self.channel.auth().project_id().clone();
        let mut channel = self.channel.clone();

        let (tx, rx) = mpsc::channel(2);

        let handle = tokio::spawn(async move {
            let mut page_token = String::new();

            let mut client = publisher_client::PublisherClient::new(&mut channel);
            loop {
                let request = pubsub::ListTopicsRequest {
                    project: format!("projects/{project_id}"),
                    page_size: 1000,
                    page_token: page_token.split_off(0),
                };

                let resp = client.list_topics(request).await?.into_inner();

                if let Err(err) = tx.send(resp.topics).await {
                    // if we cant send, the receiver, and by extension, the TopicList has been
                    // dropped. We should return the error, but there's no way to catch it, so
                    // just log it and break.
                    error!(message = "TopicList dropped while getting results", error = %err);
                    break;
                }

                if resp.next_page_token.is_empty() {
                    // if we dont have a next page token, we're done and can break.
                    break;
                } else {
                    // otherwise, set the page token and loop again.
                    page_token.clear();
                    page_token.push_str(resp.next_page_token.as_str());
                }
            }

            Ok(())
        });

        TopicList {
            rx: Some(rx),
            completed: false,
            handle,
        }
    }

    /// Shortcut for:
    ///
    /// ```no_run
    /// let client: PubSubClient = // ...
    ///
    /// client.topic("topic-name").delete().await
    /// ```
    pub async fn delete_topic<S>(&self, topic: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        self.topic(topic.as_ref()).delete().await
    }
}

pin_project_lite::pin_project! {
    pub struct TopicList {
        rx: Option<Receiver<Vec<Topic>>>,
        // since JoinHandle will panic if polled after if was already joined, we need to keep
        // track of the completion.
        completed: bool,
        #[pin]
        handle: JoinHandle<Result<(), Error>>,
    }
}
impl Stream for TopicList {
    type Item = Result<Vec<Topic>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // check the receiver, returning if it's pending/has a chunk to return.
        if let Some(rx) = this.rx.as_mut() {
            match ready!(rx.poll_recv(cx)) {
                Some(chunk) => return Poll::Ready(Some(Ok(chunk))),
                None => *this.rx = None,
            }
        }

        // if we didnt return above, the channel is closed, so we need to join the handle
        // and check for the return value being an error.
        if !*this.completed {
            let result = ready!(this.handle.poll(cx)).map_err(Error::from).flatten();

            // if ready! didn't early return, we just joined the handle and cant poll it again.
            *this.completed = true;

            // check the result
            if let Err(err) = result {
                return Poll::Ready(Some(Err(err)));
            }
        }

        Poll::Ready(None)
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_list() -> Result<(), Error> {
    use futures::StreamExt;

    let client = PubSubClient::new().await?;

    let mut list = vec![];
    let mut stream = client.list_topics();

    while let Some(result) = stream.next().await {
        let mut chunk = result?;
        list.append(&mut chunk);
    }

    println!("{list:#?}");

    Ok(())
}
