use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context, Poll};

use apache_avro::Schema;
use futures::stream::{FuturesUnordered, Stream, StreamExt};
use gcp_auth_channel::Scope;
use protos::bigquery_storage::big_query_read_client::BigQueryReadClient;
use protos::bigquery_storage::{self, ReadRowsRequest};
use rand::seq::SliceRandom;
use serde::de::DeserializeSeed;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::task::JoinHandle;

use super::stream::ReadStream;
use crate::storage::{BigQueryStorageClient, Error};

#[derive(Debug)]
pub struct ReadSession<S> {
    pub(super) read_session: bigquery_storage::ReadSession,
    pub(super) schema: Arc<Schema>,
    pub(super) client: BigQueryStorageClient,
    pub(super) seed: S,
}

impl<S> ReadSession<S>
where
    for<'de> S: DeserializeSeed<'de> + Clone,
{
    pub async fn take_stream(&mut self) -> Result<Option<ReadStream<S>>, Error> {
        let read_stream = match self.read_session.streams.pop() {
            Some(rs) => rs,
            None => return Ok(None),
        };

        let request = ReadRowsRequest {
            read_stream: read_stream.name,
            offset: 0,
        };

        let mut channel = self
            .client
            .channel
            .clone()
            .with_scope(Scope::BigQueryReadOnly);

        let row_stream = BigQueryReadClient::new(&mut channel)
            .read_rows(request)
            .await?
            .into_inner();

        Ok(Some(ReadStream::new_with_seed(
            self.read_session.streams.len(),
            row_stream,
            self.schema.clone(),
            self.seed.clone(),
        )))
    }

    pub fn into_read_streams(
        self,
    ) -> FuturesUnordered<impl Future<Output = Result<ReadStream<S>, Error>>> {
        let streams = FuturesUnordered::new();

        for (id, stream) in self.read_session.streams.into_iter().enumerate() {
            let mut channel = self
                .client
                .channel
                .clone()
                .with_scope(Scope::BigQueryReadOnly);
            let seed = self.seed.clone();
            let schema = self.schema.clone();

            streams.push(async move {
                let request = ReadRowsRequest {
                    read_stream: stream.name,
                    offset: 0,
                };

                let row_stream = BigQueryReadClient::new(&mut channel)
                    .read_rows(request)
                    .await?
                    .into_inner();

                Ok(ReadStream::new_with_seed(id, row_stream, schema, seed))
            });
        }

        streams
    }

    pub async fn spawn_stream_all<O>(self) -> Result<SpawnStreamAll<O>, Error>
    where
        for<'de> S: DeserializeSeed<'de, Value = O> + Send + 'static,
        O: Send + 'static,
    {
        let mut read_streams = self.into_read_streams();

        let handles = FuturesUnordered::new();
        let (tx, rx) = mpsc::unbounded_channel();

        while let Some(read_stream_result) = read_streams.next().await {
            let mut stream = read_stream_result?;
            let stream_tx = tx.clone();

            handles.push(tokio::spawn(async move {
                while let Some(result) = stream.next().await {
                    stream_tx.send(result)?;
                }

                Ok(()) as Result<(), Error>
            }));
        }

        Ok(SpawnStreamAll {
            handles,
            rx,
            rx_closed: false,
        })
    }

    pub async fn stream_all(self) -> Result<StreamRows<S>, Error> {
        let mut streams = Vec::with_capacity(self.read_session.streams.len());

        let mut read_streams = self.into_read_streams();

        while let Some(read_stream_result) = read_streams.next().await {
            let stream = read_stream_result?;
            streams.push(StreamState::new(stream));
        }

        Ok(StreamRows {
            streams,
            shuffle_rng: rand::thread_rng(),
        })
    }
}

#[derive(Debug)]
struct StreamState<S> {
    stream: ReadStream<S>,
    completed: bool,
}

impl<S> StreamState<S> {
    fn new(stream: ReadStream<S>) -> Self {
        Self {
            stream,
            completed: false,
        }
    }
}

impl<S> StreamRows<S> {
    pub async fn next_batch<O>(&mut self) -> Result<Option<Vec<O>>, Error>
    where
        for<'de> S: DeserializeSeed<'de, Value = O> + Clone,
    {
        self.next().await.transpose()
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct StreamRows<S> {
        streams: Vec<StreamState<S>>,
        shuffle_rng: rand::rngs::ThreadRng,
    }
}

impl<S, O> Stream for StreamRows<S>
where
    for<'de> S: DeserializeSeed<'de, Value = O> + Clone,
{
    type Item = Result<Vec<O>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // shuffle the streams around, that way we dont preferentially poll the first elements
        this.streams.shuffle(this.shuffle_rng);

        let mut all_finished = true;
        for stream in this.streams.iter_mut() {
            match stream.stream.poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => {
                    if stream.completed {
                        error!("stream previously yielded None, but just yielded Some!");
                    }
                    return Poll::Ready(Some(item));
                }
                Poll::Ready(None) => stream.completed = true,
                Poll::Pending => all_finished = false,
            }
        }

        if all_finished {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct SpawnStreamAll<O> {
        handles: FuturesUnordered<JoinHandle<Result<(), Error>>>,
        rx: UnboundedReceiver<Result<Vec<O>, Error>>,
        rx_closed: bool,
    }
}

impl<O> SpawnStreamAll<O> {
    pub async fn next_batch(&mut self) -> Result<Option<Vec<O>>, Error> {
        self.next().await.transpose()
    }
}

impl<O> Stream for SpawnStreamAll<O> {
    type Item = Result<Vec<O>, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if !*this.rx_closed {
            match ready!(this.rx.poll_recv(cx)) {
                Some(item) => return Poll::Ready(Some(item)),
                None => *this.rx_closed = true,
            }
            info!("stream rx closed");
        }

        loop {
            match ready!(this.handles.poll_next_unpin(cx)) {
                Some(Ok(Ok(_))) => info!("joined task handle ({} remaining)", this.handles.len()),
                Some(Ok(Err(err))) => return Poll::Ready(Some(Err(err))),
                Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                None => return Poll::Ready(None),
            }
        }
    }
}
