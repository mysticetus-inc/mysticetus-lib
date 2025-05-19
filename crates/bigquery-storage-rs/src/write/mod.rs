use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, PoisonError, RwLock};

use bytes::BytesMut;
use gcp_auth_channel::channel::headers::{Http, WithHeaders};
use gcp_auth_channel::{AuthChannel, Scope};
use http::HeaderValue;
use net_utils::bidirec::{self, Bidirec};
use protos::bigquery_storage::append_rows_request::{MissingValueInterpretation, ProtoData, Rows};
use protos::bigquery_storage::append_rows_response::{AppendResult, Response};
use protos::bigquery_storage::big_query_write_client::BigQueryWriteClient;
use protos::bigquery_storage::{
    self, AppendRowsRequest, AppendRowsResponse, FinalizeWriteStreamRequest, ProtoRows,
    ProtoSchema, WriteStream,
};
use tonic::transport::Channel;

use super::{BigQueryStorageClient, Error};

mod append_rows;
mod builder;
mod default;
mod schema;
mod stream_types;
mod value;
mod write2;

pub(crate) use schema::{FieldInfo, Schema};
pub use stream_types::{Buffered, Committed, Default, Pending, PendingStream};

use super::proto::ProtoSerializer;

#[derive(Debug, Clone)]
pub struct WriteClient(BigQueryStorageClient);

impl From<BigQueryStorageClient> for WriteClient {
    fn from(client: BigQueryStorageClient) -> Self {
        Self(client)
    }
}

impl WriteClient {
    /// Builds a write client, internally building a [`BigQueryStorageClient`].
    pub async fn new(project_id: &'static str, scope: Scope) -> Result<Self, Error> {
        BigQueryStorageClient::new(project_id, scope)
            .await
            .map(Self)
    }

    /// Shortcut to 'client.session_builder().dataset_id(dataset_id)'.
    pub fn dataset_id<D>(&self, dataset_id: D) -> builder::WriteSessionBuilder<(), D, ()>
    where
        D: fmt::Display,
    {
        self.session_builder().dataset_id(dataset_id)
    }

    pub fn session_builder(&self) -> builder::WriteSessionBuilder<(), (), ()> {
        builder::WriteSessionBuilder::new(self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct WriteSession<W, R> {
    inner: Arc<WriteSessionInner>,
    channel: AuthChannel<WithHeaders<Channel, Http>>,
    #[allow(dead_code)]
    // stream_type is currently only used as a marker, but that may change in the future.
    stream_type: W,
    _row_type_marker: std::marker::PhantomData<R>,
}

impl<W, R> WriteSession<W, R> {
    fn new_inner(
        mut write_stream: WriteStream,
        channel: AuthChannel,
        stream_type: W,
        schema: Option<Schema>,
    ) -> Result<Self, Error> {
        let header_str = format!("write_stream={}", write_stream.name);
        let stream_header = HeaderValue::from_str(&header_str)?;

        let channel = channel.wrap_service(|svc| {
            WithHeaders::new(svc, [(super::GOOG_REQ_PARAMS_KEY, stream_header)])
        });

        let schema = match schema {
            Some(schema) => schema,
            None => {
                let table_schema = write_stream.table_schema.take().ok_or_else(|| {
                    Error::Internal(crate::error::InternalError::NoSchemaReturned)
                })?;

                Schema::from_table_schema(table_schema)?
            }
        };

        Ok(Self {
            inner: Arc::new(WriteSessionInner {
                write_stream,
                // generate a unique trace id for this write session
                trace: uuid::Uuid::new_v4().to_string(),
                offset: AtomicUsize::new(0),
                schema: RwLock::new(schema),
            }),
            channel,
            stream_type,
            _row_type_marker: std::marker::PhantomData,
        })
    }
}

mod offset {
    use std::sync::atomic::AtomicI64;
    use std::sync::atomic::Ordering::SeqCst;

    #[derive(Debug)]
    pub struct OffsetTracker {
        last_commit: AtomicI64,
        last_append: AtomicI64,
    }

    impl Default for OffsetTracker {
        fn default() -> Self {
            Self::new()
        }
    }

    impl OffsetTracker {
        pub fn new() -> Self {
            Self {
                last_commit: AtomicI64::new(0),
                last_append: AtomicI64::new(0),
            }
        }

        pub fn get_pending_offset(&self) -> Option<i64> {
            let appended = self.append_offset();
            let committed = self.commit_offset();

            if appended > committed {
                return Some(appended);
            }

            None
        }

        pub fn append_offset(&self) -> i64 {
            self.last_append.load(SeqCst)
        }

        pub fn set_append_offset(&self, new: i64) -> i64 {
            self.last_append.fetch_max(new, SeqCst)
        }

        pub fn commit_offset(&self) -> i64 {
            self.last_commit.load(SeqCst)
        }

        pub fn set_commit_offset(&self, new: i64) -> i64 {
            self.last_commit.fetch_max(new, SeqCst)
        }
    }
}

/// Bundles up all of the shared data to put behind a single [`Arc`].
#[derive(Debug)]
struct WriteSessionInner {
    write_stream: WriteStream,
    trace: String,
    schema: RwLock<Schema>,
    offset: AtomicUsize,
}

impl<W, R> WriteSession<W, R>
where
    R: serde::Serialize,
{
    async fn get_row_append_context(
        &self,
    ) -> Result<Bidirec<AppendRowsRequest, AppendRowsResponse>, Error> {
        let mut client = BigQueryWriteClient::new(self.channel.clone());

        let (req, partial) = bidirec::build_parts();

        let handle = partial.try_initialize(client.append_rows(req)).await?;

        Ok(handle)
    }

    pub(crate) fn schema(&self) -> std::sync::RwLockReadGuard<'_, Schema> {
        self.inner
            .schema
            .read()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn handle_resp_inner(&self, resp: AppendRowsResponse) -> Result<(), Error> {
        if let Some(new_schema) = resp.updated_schema {
            let new = Schema::from_table_schema(new_schema)?;

            let mut guard = match self.inner.schema.write() {
                Ok(guard) => guard,
                Err(pois) => pois.into_inner(),
            };

            *guard = new;
        }

        match resp.response {
            Some(Response::AppendResult(AppendResult {
                offset: Some(offset),
            })) => {
                self.inner
                    .offset
                    .store(offset.value as usize, std::sync::atomic::Ordering::SeqCst);
            }
            Some(Response::Error(status)) => {
                Error::try_from_raw_status(status)?;
            }
            _ => (),
        }

        Ok(())
    }

    /// Takes an [`IntoIterator`] of rows, serializing them in order and appends them to the table.
    ///
    /// This is a placeholder for the real implementation, so there's no checks that
    /// the request payload is under the 10MB limit. Google returns a very undescriptive error
    /// when the payload is >10MB, so there's a manual check that'll error out if all of the
    /// serialized rows go over a slightly reduced limit (8MB). In a future version this'll be
    /// replaced by a unified [`append::RowAppendContext`]-based implementation that'll handle
    /// splitting the payload up internally.
    pub async fn append_rows<I>(&self, rows: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = R>,
        I::IntoIter: Send + 'static,
    {
        /*
        let mut client = BigQueryWriteClient::new(self.channel.clone());

        let base_msg = AppendRowsRequest {
            write_stream: self.inner.write_stream.name.clone(),
            ..AppendRowsRequest::default()
        };

        let encoder = append_row_encoder::AppendRowsEncoder::new_iter(
            Arc::clone(&self.inner),
            base_msg,
            rows,
        );

        let (handle, sink) = net_utils::infallible::into_infallible(futures::stream::iter(encoder));

        let mut stream = client.append_rows(sink).await?.into_inner();

        let mut last_message = None;
        while let Some(message) = stream.message().await? {
            last_message = Some(message);
        }

        let last_message = last_message
            .ok_or_else(|| Error::Status(tonic::Status::internal("append_rows never responded")))?;

        self.handle_resp_inner(last_message)
        */
        todo!()
    }
}
