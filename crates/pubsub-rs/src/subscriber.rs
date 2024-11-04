use std::sync::Arc;
use std::time::Duration;

use gcp_auth_channel::channel::AuthChannel;
use net_utils::bidirec::Bidirec;
use protos::pubsub::subscriber_client::SubscriberClient;
use protos::pubsub::{self, StreamingPullRequest, StreamingPullResponse, Subscription};
use tonic::Code;

use super::Error;

#[derive(Clone)]
pub struct Subscriber {
    sub_name: Arc<str>,
    client_id: Arc<str>,
    subscription: Subscription,
    channel: AuthChannel,
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct AckId(String);

pub struct PulledMessage {
    ack_id: Option<AckId>,
    message: pubsub::PubsubMessage,
    delivery_attempt: u32,
}

// assert that PulledMessage and pubsub::RecievedMessage
// are the same layout, that way we can efficiently turn a vec of one into a vec of another
// (that way it can reuse the buffer without reallocating)
const _: () = {
    use std::alloc::Layout;

    const A: Layout = Layout::new::<PulledMessage>();
    const B: Layout = Layout::new::<pubsub::ReceivedMessage>();

    if A.size() != B.size() {
        panic!("PulledMessage and pubsub::RecievedMessage have different sizes");
    }

    if A.align() != B.align() {
        panic!("PulledMessage and pubsub::RecievedMessage have different alignments");
    }
};

impl PulledMessage {
    fn new(raw: pubsub::ReceivedMessage) -> Option<Self> {
        let pubsub::ReceivedMessage {
            ack_id,
            message,
            delivery_attempt,
        } = raw;
        let message = message?;

        let ack_id = if ack_id.is_empty() {
            None
        } else {
            Some(AckId(ack_id))
        };

        Some(PulledMessage {
            ack_id,
            message,
            delivery_attempt: delivery_attempt as u32,
        })
    }
}

impl Subscriber {
    pub fn project_id(&self) -> &str {
        self.channel.auth().project_id()
    }

    fn client(&self) -> SubscriberClient<AuthChannel> {
        SubscriberClient::new(self.channel.clone())
    }

    pub fn subscription(&self) -> &Subscription {
        &self.subscription
    }

    pub(crate) fn new(channel: AuthChannel, subscription: Subscription) -> Self {
        let id = uuid::Uuid::new_v4();
        let mut buf = [0; uuid::fmt::Hyphenated::LENGTH];
        let client_id: &str = id.hyphenated().encode_lower(&mut buf);
        let client_id = Arc::from(client_id);

        Self {
            sub_name: Arc::from(subscription.name.as_str()),
            subscription,
            client_id,
            channel,
        }
    }

    pub async fn ack_messages_iter<I>(&self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = AckId>,
    {
        self.ack_messages(iter.into_iter().collect()).await
    }

    pub async fn ack_messages(&self, ids: Vec<AckId>) -> Result<(), Error> {
        let req = pubsub::AcknowledgeRequest {
            // SAFETY: AckId is repr transparent
            ack_ids: unsafe { std::mem::transmute(ids) },
            subscription: self.subscription.name.clone(),
        };

        self.client().acknowledge(req).await?;

        Ok(())
    }

    pub async fn pull(&self, max_message_count: usize) -> Result<Vec<PulledMessage>, Error> {
        assert_ne!(max_message_count, 0, "must be non-zero");

        #[allow(deprecated)] // for 'return_immediately'
        let req = pubsub::PullRequest {
            subscription: self.subscription.name.clone(),
            max_messages: max_message_count as i32,
            // the struct needs to be fully populated, but I also don't want include a wildcard
            // ..Default::default(), since then new fields will be implicitely filled in
            // too. Would rather have new fields show as a compile time error.
            return_immediately: Default::default(),
        };

        let resp = self.client().pull(req).await?.into_inner();

        Ok(resp
            .received_messages
            .into_iter()
            .filter_map(PulledMessage::new)
            .collect())
    }

    pub async fn streaming_pull(
        &self,
        ack_deadline: Duration,
        max_outstanding_messages: usize,
    ) -> Result<StreamingPull, Error> {
        let (req, partially_init) = Bidirec::parts();

        let mut client = self.client();

        let handle = partially_init
            .try_initialize(client.streaming_pull(req))
            .await?;

        let mut pull = StreamingPull {
            sub_name: Arc::clone(&self.sub_name),
            client_id: Arc::clone(&self.client_id),
            ack_deadline,
            max_outstanding_messages,
            handle,
            client,
        };

        pull.send_initial_message()?;

        Ok(pull)
    }
}

pub struct StreamingPull {
    handle: Bidirec<StreamingPullRequest, StreamingPullResponse>,
    // needed to restart the streaming pull if a crash is encountered,
    client: SubscriberClient<AuthChannel>,
    sub_name: Arc<str>,
    client_id: Arc<str>,
    ack_deadline: Duration,
    max_outstanding_messages: usize,
}

impl StreamingPull {
    fn send_initial_message(&mut self) -> Result<(), Error> {
        let first_message = pubsub::StreamingPullRequest {
            subscription: self.sub_name.as_ref().to_owned(),
            ack_ids: Vec::new(),
            modify_deadline_seconds: Vec::new(),
            modify_deadline_ack_ids: Vec::new(),
            client_id: self.client_id.as_ref().to_owned(),
            stream_ack_deadline_seconds: self.ack_deadline.as_secs_f64().round() as i32,
            max_outstanding_messages: self.max_outstanding_messages as i64,
            max_outstanding_bytes: 0,
        };

        self.handle
            .send(first_message)
            .map_err(|_| Error::Internal("streaming pull driver crashed"))
    }

    async fn reinit(&mut self) -> Result<(), tonic::Status> {
        self.handle.close();

        let (req, partially_init) = Bidirec::parts();

        self.handle = partially_init
            .try_initialize(self.client.streaming_pull(req))
            .await?;

        Ok(())
    }

    pub fn shutdown(mut self) -> usize {
        self.handle.close();
        self.handle.pending_messages()
    }

    async fn next_internal(&mut self) -> Result<Vec<PulledMessage>, Result<tonic::Status, Error>> {
        let mut retries_left = 5_u8;

        let resp = loop {
            match self.handle.message().await.map_err(Ok)? {
                Some(resp) => break resp,
                None => {
                    self.reinit().await.map_err(Ok)?;
                    self.send_initial_message().map_err(Err)?;
                }
            }

            retries_left = match retries_left.checked_sub(1) {
                Some(next) => next,
                None => return Err(Err(Error::Internal("can't restart streaming_pull"))),
            };
        };

        println!("{:#?}", resp.subscription_properties);

        let messages = resp
            .received_messages
            .into_iter()
            .filter_map(PulledMessage::new)
            .collect::<Vec<_>>();
        Ok(messages)
    }

    pub async fn next_batch(&mut self) -> Result<Vec<PulledMessage>, Error> {
        let mut retries_left = 5_u8;

        loop {
            match self.next_internal().await {
                Ok(messages) => return Ok(messages),
                Err(Err(hard_error)) => return Err(hard_error),
                // retry on unavailable
                Err(Ok(status)) if status.code() == Code::Unavailable => {
                    self.reinit().await?;
                    self.send_initial_message()?;
                }
                Err(Ok(invalid_status)) => return Err(Error::Status(invalid_status)),
            }

            retries_left = match retries_left.checked_sub(1) {
                Some(next) => next,
                None => return Err(Error::Internal("can't restart streaming_pull")),
            };
        }
    }

    pub fn ack_messages_iter<I>(&self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = AckId>,
    {
        self.ack_messages(iter.into_iter().collect())
    }

    pub fn ack_messages(&self, ids: Vec<AckId>) -> Result<(), Error> {
        let req = pubsub::StreamingPullRequest {
            subscription: self.sub_name.as_ref().to_owned(),
            // SAFETY: AckId is repr transparent
            ack_ids: unsafe { std::mem::transmute(ids) },
            ..Default::default()
        };

        self.handle
            .send(req)
            .map_err(|_| Error::Internal("streaming_pull driver crashed"))
    }
}

impl Drop for StreamingPull {
    fn drop(&mut self) {
        if !self.handle.is_closed() {
            self.handle.close();
        }
    }
}
