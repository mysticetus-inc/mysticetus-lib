use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use gcp_auth_channel::channel::AuthChannel;
use protos::pubsub::{self, PubsubMessage, Topic, publisher_client};

use super::Error;
use super::util::make_default_topic;

const MAX_PER_REQUEST: usize = 1000;

#[derive(Debug, Clone)]
pub struct TopicClient {
    topic: Arc<Topic>,
    channel: AuthChannel,
}

impl TopicClient {
    pub(crate) fn new_from_topic<T>(topic: T, channel: AuthChannel) -> Self
    where
        T: Into<Arc<Topic>>,
    {
        Self {
            topic: topic.into(),
            channel,
        }
    }

    pub(crate) fn new_from_name<S>(topic_name: S, channel: AuthChannel) -> Self
    where
        S: AsRef<str>,
    {
        let topic = make_default_topic(channel.auth().project_id(), topic_name.as_ref());
        Self::new_from_topic(Arc::new(topic), channel)
    }

    pub fn project_id(&self) -> &str {
        self.channel.auth().project_id()
    }

    pub fn topic(&self) -> &Topic {
        &self.topic
    }

    pub async fn delete(mut self) -> Result<(), Error> {
        debug!(
            message = "deleting topic...",
            topic_name = self.topic.name.as_str()
        );

        let request = pubsub::DeleteTopicRequest {
            topic: self.topic.name.as_str().to_owned(),
        };

        publisher_client::PublisherClient::new(&mut self.channel)
            .delete_topic(request)
            .await?;

        debug!(message = "successfully deleted topic");

        Ok(())
    }

    async fn publish_inner(&mut self, messages: Vec<PubsubMessage>) -> Result<Vec<String>, Error> {
        let message_count = messages.len();

        if message_count == 0 {
            info!("no messages to publish, skipping request");
            return Ok(Vec::new());
        }

        debug!(
            message = "publishing messages...",
            count = message_count,
            topic = self.topic.name.as_str(),
        );

        let request = pubsub::PublishRequest {
            topic: self.topic.name.clone(),
            messages,
        };

        let mut client = publisher_client::PublisherClient::new(&mut self.channel);

        let response = loop {
            let mut backoff: Option<net_utils::backoff::Backoff> = None;

            let error = match client.publish(request.clone()).await {
                Ok(resp) => break resp.into_inner(),
                Err(error) if error.code() == tonic::Code::Internal => error,
                Err(other_error) => return Err(other_error.into()),
            };

            match backoff.get_or_insert_default().backoff_once() {
                Some(backoff) => {
                    warn!(
                        message = "sending messages failed, backing off",
                        ?backoff,
                        ?error
                    );
                    backoff.await;
                }
                None => {
                    error!(
                        message = "pubsub: sending messages hit maximum number of retries",
                        ?error
                    );
                    return Err(error.into());
                }
            }
        };

        let id_count = response.message_ids.len();

        if message_count != id_count {
            warn!(
                message = "mismatched number of ids recieved",
                id_count,
                message_count,
                topic = self.topic.name.as_str(),
            );
        } else {
            debug!(
                message = "successfully published messages",
                id_count,
                topic = self.topic.name.as_str(),
            );
        }

        Ok(response.message_ids)
    }

    pub async fn publish_serialized_message(
        &mut self,
        serialized: Vec<u8>,
    ) -> Result<String, Error> {
        let message = PubsubMessage {
            data: serialized.into(),
            attributes: HashMap::new(),
            message_id: String::new(),
            publish_time: None,
            ordering_key: String::new(),
        };

        let mut ids = self.publish_inner(vec![message]).await?;

        if ids.len() != 1 {
            warn!(
                message = "expected 1 message id in response",
                found = ids.len(),
                topic = self.topic.name.as_str(),
            );
        }

        ids.pop()
            .ok_or_else(|| Error::Internal("expected to recieve 1 message id in response"))
    }

    pub async fn publish_message<M>(&mut self, message: &M) -> Result<String, Error>
    where
        M: serde::Serialize,
    {
        let data = serde_json::to_vec(message)?;
        self.publish_serialized_message(data).await
    }

    pub async fn publish_messages<'a, I, M>(&mut self, iter: I) -> Result<Vec<String>, Error>
    where
        I: IntoIterator<Item = &'a M>,
        M: serde::Serialize + 'a,
    {
        let iter = iter.into_iter();
        let size_hint = match iter.size_hint() {
            (_, Some(high)) => high,
            (low, None) => low,
        };

        let mut messages = Vec::with_capacity(size_hint);

        for message in iter.into_iter() {
            let data = serde_json::to_vec(message)?.into();

            messages.push(PubsubMessage {
                data,
                attributes: HashMap::new(),
                message_id: String::new(),
                publish_time: None,
                ordering_key: String::new(),
            });
        }

        self.publish_inner(messages).await
    }

    pub async fn publish_serialized_messages<I>(&mut self, iter: I) -> Result<Vec<String>, Error>
    where
        I: IntoIterator,
        Bytes: From<I::Item>,
    {
        let iter = iter.into_iter();
        let size_hint = match iter.size_hint() {
            (_, Some(high)) => high,
            (low, None) => low,
        };

        let mut messages = Vec::with_capacity(size_hint);

        for data in iter.into_iter() {
            messages.push(PubsubMessage {
                data: Bytes::from(data),
                ..Default::default()
            });
        }

        self.publish_inner(messages).await
    }

    pub fn batch_publish(&mut self) -> BatchPublishContext<'_> {
        BatchPublishContext {
            client: self,
            messages: Vec::new(),
        }
    }
}

pub struct BatchPublishContext<'a> {
    client: &'a mut TopicClient,
    messages: Vec<PubsubMessage>,
}

impl BatchPublishContext<'_> {
    pub fn add_message<M>(&mut self, message: &M) -> Result<(), Error>
    where
        M: serde::Serialize,
    {
        let data = serde_json::to_vec(message)?;

        self.add_serialized_message(data);

        Ok(())
    }

    pub fn add_serialized_message(&mut self, message: Vec<u8>) {
        self.messages.push(PubsubMessage {
            data: message.into(),
            attributes: HashMap::new(),
            message_id: String::new(),
            publish_time: None,
            ordering_key: String::new(),
        });
    }

    pub fn add_messages<'a, I, M>(&mut self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a M>,
        M: serde::Serialize + 'a,
    {
        for message in iter.into_iter() {
            self.add_message(message)?;
        }

        Ok(())
    }

    pub fn add_serialized_messages<I>(&mut self, iter: I)
    where
        I: IntoIterator,
        Bytes: From<I::Item>,
    {
        let iter = iter.into_iter();
        let size_hint = match iter.size_hint() {
            (_, Some(high)) => high,
            (low, None) => low,
        };

        self.messages.reserve(size_hint);

        self.messages.extend(iter.map(|data| PubsubMessage {
            data: Bytes::from(data),
            ..Default::default()
        }));
    }

    pub async fn publish(mut self) -> Result<Vec<String>, Error> {
        let mut ids = Vec::new();
        while self.messages.len() > MAX_PER_REQUEST {
            let batch = self
                .messages
                .split_off(self.messages.len() - MAX_PER_REQUEST);

            let mut batch_ids = self.client.publish_inner(batch).await?;
            ids.append(&mut batch_ids);
        }

        let mut last_ids = self.client.publish_inner(self.messages).await?;

        if ids.is_empty() {
            Ok(last_ids)
        } else {
            ids.append(&mut last_ids);
            Ok(ids)
        }
    }
}

impl Extend<Vec<u8>> for BatchPublishContext<'_> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        self.add_serialized_messages(iter);
    }
}

impl Extend<Bytes> for BatchPublishContext<'_> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Bytes>,
    {
        self.add_serialized_messages(iter);
    }
}
