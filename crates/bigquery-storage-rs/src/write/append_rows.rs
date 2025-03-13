use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, PoisonError};
use std::task::{Context, Poll};

use bytes::{Bytes, BytesMut};
use futures::Stream;
use net_utils::bidi2::{self, RequestSink};
use prost::Message;
use protos::bigquery_storage::append_rows_request::{
    self, MissingValueInterpretation, ProtoData, Rows,
};
use protos::bigquery_storage::big_query_write_client::BigQueryWriteClient;
use protos::bigquery_storage::{AppendRowsRequest, AppendRowsResponse, ProtoRows};
use protos::protobuf::DescriptorProto;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::schema::Schema;
use crate::error::InternalError;
use crate::proto::ProtoSerializer;

pub struct AppendRowsRequestIter<'a, W, R> {
    write_session: &'a mut super::WriteSession<W, R>,
    sink: RequestSink<AppendRowsRequest>,
    driver: JoinHandle<crate::Result<()>>,
    response_channel_tx: UnboundedSender<oneshot::Sender<crate::Result<()>>>,
    row_iter: Option<R>,
    next: Option<AppendRowsRequest>,
}

#[derive(Debug, Clone, Default)]
pub struct MissingValueInterpretations {
    pub(crate) per_field: HashMap<String, i32>,
    pub(crate) default: MissingValueInterpretation,
}

impl<'a, W, R> AppendRowsRequestIter<'a, W, R>
where
    R: Iterator,
    R::Item: serde::Serialize,
{
    pub(super) async fn new(
        write_session: &'a mut super::WriteSession<W, R>,
        row_iter: R,
        proto_descriptor: DescriptorProto,
        missing_value_interpretations: MissingValueInterpretations,
    ) -> crate::Result<Self> {
        let (sink, requests) = bidi2::build_pair();
        let responses = BigQueryWriteClient::new(&mut write_session.channel)
            .append_rows(requests)
            .await?
            .into_inner();

        let (response_channel_tx, response_txs) = mpsc::unbounded_channel();

        let driver = tokio::spawn(DriverFuture {
            write_stream_inner: Arc::clone(&write_session.inner),
            response_txs: Some(response_txs),
            responses,
            pending: VecDeque::with_capacity(16),
        });

        let first_request = AppendRowsRequest {
            write_stream: write_session.inner.write_stream.name.clone(),
            offset: None,
            trace_id: write_session.inner.trace.clone(),
            missing_value_interpretations: missing_value_interpretations.per_field,
            default_missing_value_interpretation: missing_value_interpretations.default as i32,
            rows: Some(Rows::ProtoRows(ProtoData {
                writer_schema: Some(protos::bigquery_storage::ProtoSchema {
                    proto_descriptor: Some(proto_descriptor),
                }),
                rows: None,
            })),
        };

        Ok(Self {
            write_session,
            row_iter: Some(row_iter),
            driver,
            sink,
            next: Some(first_request),
            response_channel_tx,
        })
    }

    fn fill_row_buf(&mut self) -> crate::Result<usize> {
        let Some(ref mut row_iter) = self.row_iter else {
            return Ok(0);
        };

        #[inline]
        fn get_row_buf(
            message: &mut AppendRowsRequest,
            size_hint: impl FnOnce() -> (usize, Option<usize>),
        ) -> &mut Vec<Bytes> {
            let rows = match message.rows {
                Some(ref mut rows) => rows,
                None => message.rows.insert(Rows::ProtoRows(ProtoData {
                    writer_schema: None,
                    rows: None,
                })),
            };

            let rows = match rows {
                Rows::ProtoRows(rows) => &mut rows.rows,
                _ => unreachable!("we never use arrow rows"),
            };

            match rows {
                Some(rows) => &mut rows.serialized_rows,
                None => {
                    let (low, high) = size_hint();
                    let serialized_rows = Vec::with_capacity(high.unwrap_or(low).min(64));
                    &mut rows.insert(ProtoRows { serialized_rows }).serialized_rows
                }
            }
        }

        let next_message = self.next.get_or_insert_with(|| AppendRowsRequest {
            write_stream: self.write_session.inner.write_stream.name.clone(),
            offset: None,
            trace_id: self.write_session.inner.trace.clone(),
            missing_value_interpretations: HashMap::new(),
            default_missing_value_interpretation:
                append_rows_request::MissingValueInterpretation::Unspecified as i32,
            rows: Some(Rows::ProtoRows(ProtoData {
                rows: None,
                writer_schema: None,
            })),
        });

        let mut last_encoded_len = next_message.encoded_len();
        let mut max_encoded_row_size = 0;

        let schema = self
            .write_session
            .inner
            .schema
            .read()
            .unwrap_or_else(PoisonError::into_inner);

        while last_encoded_len + max_encoded_row_size < 10 * 1024 * 1024 {
            let Some(row) = row_iter.next() else {
                self.row_iter = None;
                return Ok(get_row_buf(next_message, || (0, Some(0))).len());
            };

            let mut buf = BytesMut::with_capacity(max_encoded_row_size);
            ProtoSerializer::new(&mut buf, &schema).serialize_row(&row)?;

            max_encoded_row_size = max_encoded_row_size.max(buf.len());

            get_row_buf(next_message, || row_iter.size_hint()).push(buf.freeze());

            last_encoded_len = next_message.encoded_len();
        }

        Ok(get_row_buf(next_message, || (0, Some(0))).len())
    }
}

// TODO: maybe use stream instead of iterator

impl<'a, W, R> Iterator for AppendRowsRequestIter<'a, W, R>
where
    R: Iterator,
    R::Item: serde::Serialize,
{
    type Item = crate::Result<PendingRequest>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_none() && self.row_iter.is_none() {
            return None;
        }

        match self.fill_row_buf() {
            Ok(0) => return None,
            Ok(_) => {
                let request = self
                    .next
                    .take()
                    .expect("fill_row_buf will set this if self.row_iter is Some");

                let (tx, rx) = oneshot::channel();

                if self.sink.send(request).is_err() || self.response_channel_tx.send(tx).is_err() {
                    return todo!();
                }

                return Some(Ok(PendingRequest { rx }));
            }
            Err(error) => return Some(Err(error)),
        }
    }
}

pub struct PendingRequest {
    // approx, not actual type
    rx: oneshot::Receiver<crate::Result<()>>,
}

pin_project_lite::pin_project! {
    #[project = DriverFutureProjection]
    pub struct DriverFuture {
        response_txs: Option<UnboundedReceiver<oneshot::Sender<crate::Result<()>>>>,
        #[pin]
        responses: tonic::Streaming<AppendRowsResponse>,
        pending: VecDeque<oneshot::Sender<crate::Result<()>>>,
        write_stream_inner: Arc<super::WriteSessionInner>,
    }
}

impl DriverFutureProjection<'_> {
    fn poll_response_txs(&mut self, cx: &mut Context<'_>) {
        let Some(response_txs) = self.response_txs else {
            return;
        };

        loop {
            match response_txs.poll_recv(cx) {
                Poll::Pending => return,
                Poll::Ready(None) => {
                    *self.response_txs = None;
                    return;
                }
                Poll::Ready(Some(msg)) => self.pending.push_back(msg),
            }
        }
    }

    fn handle_response(&mut self, response: AppendRowsResponse) -> crate::Result<()> {
        if let Some(schema) = response.updated_schema {
            let schema = Schema::from_table_schema(schema)?;
            let mut guard = self
                .write_stream_inner
                .schema
                .write()
                .unwrap_or_else(PoisonError::into_inner);

            *guard = schema;
        }

        if let Some(response) = response.response {
            let sender = self
                .pending
                .pop_front()
                .ok_or_else(|| crate::Error::Internal(InternalError::TooManyAppendRowResponses))?;

            match response {
                protos::bigquery_storage::append_rows_response::Response::Error(error) => {
                    let _ = sender.send(Err(crate::Error::from(error)));
                }
                protos::bigquery_storage::append_rows_response::Response::AppendResult(result) => {
                    if let Some(offset) = result.offset {
                        self.write_stream_inner
                            .offset
                            .fetch_max(offset.value as usize, std::sync::atomic::Ordering::SeqCst);
                    }
                    sender.send(Ok(()));
                }
            }
        }

        Ok(())
    }
}

impl Future for DriverFuture {
    type Output = crate::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        this.poll_response_txs(cx);

        loop {
            match std::task::ready!(this.responses.as_mut().poll_next(cx)) {
                Some(result) => {
                    let message = result?;
                    if this.pending.is_empty() {
                        this.poll_response_txs(cx);
                    }
                    this.handle_response(message)?;
                }
                None => {
                    this.pending.clear();
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}
