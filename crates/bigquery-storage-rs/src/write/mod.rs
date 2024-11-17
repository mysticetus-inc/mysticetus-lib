use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use bytes::BytesMut;
use gcp_auth_channel::Scope;
use http::HeaderValue;
use net_utils::bidirec::{self, Bidirec};
use protos::bigquery_storage::append_rows_request::{MissingValueInterpretation, ProtoData, Rows};
use protos::bigquery_storage::append_rows_response::{AppendResult, Response};
use protos::bigquery_storage::big_query_write_client::BigQueryWriteClient;
use protos::bigquery_storage::{
    self, AppendRowsRequest, AppendRowsResponse, FinalizeWriteStreamRequest, ProtoRows,
    ProtoSchema, WriteStream,
};

use super::{BigQueryStorageClient, Error};

mod append;
mod builder;
mod default;
mod stream_types;

pub use stream_types::{Buffered, Committed, Default, Pending, PendingStream};

use super::proto::{ProtoSerializer, Schemas};

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
    client: BigQueryStorageClient,
    #[allow(dead_code)]
    // stream_type is currently only used as a marker, but that may change in the future.
    stream_type: W,
    _row_type_marker: std::marker::PhantomData<R>,
}

impl<W, R> WriteSession<W, R> {
    fn new_inner(
        mut write_stream: WriteStream,
        client: BigQueryStorageClient,
        stream_type: W,
    ) -> Result<Self, Error> {
        let header_str = format!("write_stream={}", write_stream.name);
        let stream_header = HeaderValue::from_str(&header_str)?;

        let table_schema = write_stream.table_schema.take().ok_or(Error::Internal(
            crate::error::InternalError::NoSchemaReturned,
        ))?;

        let schemas = Schemas::new_with_type_name::<R>(table_schema)?;

        Ok(Self {
            inner: Arc::new(WriteSessionInner {
                write_stream,
                stream_header,
                // generate a unique trace id for this write session
                trace: uuid::Uuid::new_v4().to_string(),
                offsets: offset::OffsetTracker::new(),
                schemas: Mutex::new(Arc::new(schemas)),
            }),
            client,
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

/// Bundles up all of the shared data to put behind a single [`Arc`], rather than one per field.
/// Also, not generic over any type, so any internal code can be optimized without any
/// monomorphization (as long as the serialization occurs outside of one of its methods, that
/// may change).
#[derive(Debug)]
struct WriteSessionInner {
    write_stream: WriteStream,
    trace: String,
    schemas: Mutex<Arc<Schemas>>,
    offsets: offset::OffsetTracker,
    stream_header: HeaderValue,
}

impl<W, R> WriteSession<W, R>
where
    R: serde::Serialize,
{
    async fn get_row_append_context(
        &self,
    ) -> Result<Bidirec<AppendRowsRequest, AppendRowsResponse>, Error> {
        let channel = self
            .client
            .channel
            .clone()
            .attach_header()
            .static_key(super::GOOG_REQ_PARAMS_KEY)
            .value(self.inner.stream_header.clone())
            .with_scope(Scope::BigQueryReadWrite);

        let mut client = BigQueryWriteClient::new(channel);

        let (req, partial) = bidirec::build_parts();

        let handle = partial.try_initialize(client.append_rows(req)).await?;

        Ok(handle)
    }

    pub(crate) fn get_schemas(&self) -> Arc<Schemas> {
        match self.inner.schemas.lock() {
            Ok(lock) => Arc::clone(&*lock),
            Err(poisoned) => Arc::clone(&*poisoned.into_inner()),
        }
    }

    fn handle_resp_inner(&self, resp: AppendRowsResponse) -> Result<i64, Error> {
        if let Some(new_schema) = resp.updated_schema {
            let new = Schemas::new_with_type_name::<R>(new_schema).map(Arc::new)?;

            let mut guard = match self.inner.schemas.lock() {
                Ok(guard) => guard,
                Err(pois) => pois.into_inner(),
            };

            *guard = new;
        }

        match resp.response {
            Some(Response::AppendResult(AppendResult {
                offset: Some(offset),
            })) => {
                return Ok(self.inner.offsets.set_append_offset(offset.value));
            }
            Some(Response::Error(status)) => {
                Error::try_from_raw_status(status)?;
            }
            _ => (),
        }

        Ok(self.inner.offsets.append_offset())
    }

    /// Takes an [`IntoIterator`] of rows, serializing them in order and appends them to the table.
    ///
    /// This is a placeholder for the real implementation, so there's no checks that
    /// the request payload is under the 10MB limit. Google returns a very undescriptive error
    /// when the payload is >10MB, so there's a manual check that'll error out if all of the
    /// serialized rows go over a slightly reduced limit (8MB). In a future version this'll be
    /// replaced by a unified [`append::RowAppendContext`]-based implementation that'll handle
    /// splitting the payload up internally.
    pub async fn append_rows<I>(&self, rows: I) -> Result<i64, Error>
    where
        I: IntoIterator<Item = R>,
    {
        let row_iter = rows.into_iter();
        let (low, high) = row_iter.size_hint();

        let mut rows = Vec::with_capacity(high.unwrap_or(low));

        let schemas = self.get_schemas();

        let mut row_size_sum = 0_usize;

        for row in row_iter {
            let mut row_buf = row_size_sum
                .checked_div(rows.len())
                .map(BytesMut::with_capacity)
                .unwrap_or_default();

            ProtoSerializer::new(&mut row_buf, &schemas).serialize_row(&row)?;

            row_size_sum += row_buf.len();

            rows.push(row_buf.freeze());
        }

        // BQ throws errors if it gets empty rows (plus it's doing IO for no reason), so bail early
        if rows.is_empty() {
            return Ok(0);
        }

        if row_size_sum > 8 * 1024_usize.pow(3) {
            return Err(Error::Status(tonic::Status::internal(
                "serialized rows are over the temp 8MB limit",
            )));
        }

        let req = AppendRowsRequest {
            rows: Some(Rows::ProtoRows(ProtoData {
                writer_schema: Some(ProtoSchema {
                    proto_descriptor: Some(schemas.proto().clone()),
                }),
                rows: Some(ProtoRows {
                    serialized_rows: rows,
                }),
            })),
            default_missing_value_interpretation: MissingValueInterpretation::DefaultValue as i32,
            missing_value_interpretations: HashMap::new(),
            write_stream: self.inner.write_stream.name.clone(),
            offset: None,
            trace_id: self.inner.trace.clone(),
        };

        let channel = self
            .client
            .channel
            .clone()
            .attach_header()
            .static_key(super::GOOG_REQ_PARAMS_KEY)
            .value(self.inner.stream_header.clone())
            .with_scope(Scope::BigQueryReadWrite);

        let mut client = BigQueryWriteClient::new(channel);

        let mut stream = client
            .append_rows(net_utils::once::Once::new(req))
            .await?
            .into_inner();

        let mut last_message = None;
        while let Some(message) = stream.message().await? {
            last_message = Some(message);
        }

        let last_message = last_message
            .ok_or_else(|| Error::Status(tonic::Status::internal("append_rows never responded")))?;

        self.handle_resp_inner(last_message)
    }
}

impl<W, R> WriteSession<W, R>
where
    W: stream_types::WriteStreamType<CanFlush = typenum::B1>,
{
    /// Flushes the stream, returning the offset of the flushed rows.
    pub async fn flush(&self) -> Result<Option<usize>, Error> {
        let value = match self.inner.offsets.get_pending_offset() {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let req = bigquery_storage::FlushRowsRequest {
            write_stream: self.inner.write_stream.name.clone(),
            offset: Some(protos::protobuf::Int64Value { value }),
        };

        let channel = self
            .client
            .channel
            .clone()
            .with_scope(Scope::BigQueryReadWrite);

        let mut client = BigQueryWriteClient::new(channel);

        let resp = client.flush_rows(req).await?.into_inner();

        let new_offset = self.inner.offsets.set_commit_offset(resp.offset);

        Ok(Some(new_offset.abs_diff(resp.offset) as usize))
    }
}

impl<W, R> WriteSession<W, R>
where
    W: stream_types::FinalizeStream,
{
    /// Finalizes the stream. If this is a [`Pending`] stream, the returned [`PendingStream`]
    /// must be committed.
    ///
    /// To do so from one call, use [`WriteSession::finalize_and_commit`].
    pub async fn finalize(self) -> Result<W::Ok, Error> {
        let req = FinalizeWriteStreamRequest {
            name: self.inner.write_stream.name.clone(),
        };

        let channel = self
            .client
            .channel
            .clone()
            .with_scope(Scope::BigQueryReadWrite);

        let mut client = BigQueryWriteClient::new(channel);

        let resp = client.finalize_write_stream(req).await?.into_inner();

        W::on_finalized(self, resp)
    }
}

impl<R> WriteSession<Pending, R> {
    /// Provides a shortcut for finalizing + committing a [`Pending`] stream.
    pub async fn finalize_and_commit(self) -> Result<usize, Error> {
        self.finalize().await?.commit().await
    }
}
