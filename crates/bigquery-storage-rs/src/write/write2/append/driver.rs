use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use net_utils::bidi2::{self, RequestSink};
use protos::bigquery_storage::{big_query_write_client, AppendRowsRequest, AppendRowsResponse};
use tokio::sync::{mpsc, oneshot};

use crate::write::write2::session::WriteSessionShared;

use super::super::{StreamType, WriteSession, WriteSessionState};

pub(super) type NewRequestPair = (
    AppendRowsRequest,
    oneshot::Sender<tonic::Result<AppendRowsResponse>>,
);

pin_project_lite::pin_project! {
    #[project = DriverProjection]
    pub(super) struct Driver<Type: StreamType> {
        shared: Arc<WriteSessionShared<Type>>,
        stream: tonic::Streaming<AppendRowsResponse>,
        rx: mpsc::UnboundedReceiver<NewRequestPair>,
        sink: RequestSink<AppendRowsRequest>,
        pending: VecDeque<oneshot::Sender<AppendRowsResponse>>,
    }
}

impl<Type: StreamType> Driver<Type> {
    pub(super) async fn new<R>(
        session: &WriteSession<R, Type>,
    ) -> crate::Result<(mpsc::UnboundedSender<NewRequestPair>, Self)> {
        let channel = session.channel.clone();

        let (sink, stream) = bidi2::build_pair();

        let stream = big_query_write_client::BigQueryWriteClient::new(channel)
            .append_rows(stream)
            .await?
            .into_inner();

        let (tx, rx) = mpsc::unbounded_channel();

        let driver = Self {
            shared: Arc::clone(&session.shared()),
            rx,
            stream,
            sink,
            pending: VecDeque::with_capacity(32),
        };

        Ok((tx, driver))
    }
}

impl<Type: StreamType> Future for Driver<Type> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        todo!()
    }
}
