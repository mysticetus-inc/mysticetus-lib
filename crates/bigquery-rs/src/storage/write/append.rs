use std::marker::PhantomData;
use std::sync::Arc;

use gcp_auth_channel::Scope;
use net_utils::bidirec::Bidirec;
use protos::bigquery_storage::big_query_write_client::BigQueryWriteClient;
use protos::bigquery_storage::{AppendRowsRequest, AppendRowsResponse};

use super::{Error, WriteSession, WriteSessionInner};

#[derive(Debug)]
pub struct RowAppendContext<R> {
    session: Arc<WriteSessionInner>,
    client: super::BigQueryStorageClient,
    cached: cached::CachedRows,
    handle: Bidirec<AppendRowsRequest, AppendRowsResponse>,
    _marker: PhantomData<R>,
}

impl<R> RowAppendContext<R> {
    pub(super) async fn init<W>(parent: &WriteSession<W, R>) -> Result<Self, crate::Error> {
        let (req_stream, partially_init) = net_utils::bidirec::build_parts();

        let channel = parent
            .client
            .channel
            .clone()
            .attach_header()
            .static_key(crate::storage::GOOG_REQ_PARAMS_KEY)
            .value(parent.inner.stream_header.clone())
            .with_scope(Scope::BigQueryReadWrite);

        let mut client = BigQueryWriteClient::new(channel);

        let handle = partially_init
            .try_initialize(client.append_rows(req_stream))
            .await?;

        Ok(Self {
            session: Arc::clone(&parent.inner),
            cached: cached::CachedRows::default(),
            client: parent.client.clone(),
            handle,
            _marker: PhantomData,
        })
    }

    async fn restart_stream(&mut self) -> Result<RowAppendCommitStream, crate::Error> {
        let (req_stream, partially_init) = net_utils::bidirec::build_parts();

        let channel = self
            .client
            .channel
            .clone()
            .attach_header::<gcp_auth_channel::channel::headers::Http>()
            .static_key(crate::storage::GOOG_REQ_PARAMS_KEY)
            .value(self.session.stream_header.clone())
            .with_scope(Scope::BigQueryReadWrite);

        let mut client = BigQueryWriteClient::new(channel);

        let new_handle = partially_init
            .try_initialize(client.append_rows(req_stream))
            .await?;

        let old_handle = std::mem::replace(&mut self.handle, new_handle);

        Ok(RowAppendCommitStream { handle: old_handle })
    }

    pub async fn finish(self) -> RowAppendCommitStream {
        RowAppendCommitStream {
            handle: self.handle,
        }
    }
}

#[derive(Debug)]
pub struct RowAppendCommitStream {
    handle: Bidirec<AppendRowsRequest, AppendRowsResponse>,
}

mod cached {
    use std::collections::VecDeque;

    use data_structures::circular_buffer::CircularBuffer;

    use super::Error;

    /// The BQ docs specify a 10MB max per request, but we'll only use 75% of that to make sure
    /// the [`DescriptorProto`] doesn't tip us over, since it's not quite as clear how large
    /// that'll serialize to under the hood.
    const MAX_PAYLOAD_SIZE: usize = (75 * 1024_usize.pow(3)) / 100;

    const WINDOW_SIZE: usize = 50;

    /// FIFO cache for serialized rows. Manages chunks of serialized rows for efficient insertion
    /// and removal.
    #[derive(Debug, Default)]
    pub struct CachedRows {
        chunks: VecDeque<(Vec<Vec<u8>>, usize)>,
        last_sizes: CircularBuffer<WINDOW_SIZE, usize>,
    }

    // helper macros for getting/inserting chunks infallibly.
    // these can't be individual functions, since returning a mutable reference
    // makes mutating `last_size` impossible, even though these don't touch `last_size`
    macro_rules! get_new_chunk {
        ($self:expr) => {{
            $self.chunks.push_back((Vec::new(), 0));
            $self.chunks.back_mut().unwrap()
        }};
    }

    macro_rules! get_last_chunk {
        ($self:expr) => {{
            // if we're empty, insert an empty chunk/size so we can unwrap back_mut.
            if $self.chunks.is_empty() {
                $self.chunks.push_back((Vec::new(), 0));
            }

            $self.chunks.back_mut().unwrap()
        }};
    }

    impl CachedRows {
        pub(super) fn is_empty(&self) -> bool {
            self.chunks.is_empty()
        }

        pub(crate) fn estimate_needed_row_capacity(&self) -> usize {
            let sum = self
                .last_sizes
                .as_discontinuous_slice()
                .iter()
                .sum::<usize>();

            sum / self.last_sizes.len()
        }

        pub(super) fn num_chunks(&self) -> usize {
            self.chunks.len()
        }

        pub(super) fn num_rows(&self) -> usize {
            self.chunks
                .iter()
                .map(|(chunk, _)| chunk.len())
                .sum::<usize>()
        }

        pub(super) fn serialize_rows<I, R>(
            &mut self,
            schemas: &super::super::Schemas,
            iter: I,
        ) -> Result<(), Error>
        where
            I: IntoIterator<Item = R>,
            R: serde::Serialize,
        {
            let cap = self.estimate_needed_row_capacity();

            let mut chunk = get_last_chunk!(self);

            for row in iter {
                if chunk.1 > MAX_PAYLOAD_SIZE {
                    chunk = get_new_chunk!(self);
                }

                let mut buf = Vec::with_capacity(cap);
                super::super::ProtoSerializer::new(&mut buf, schemas).serialize_row(&row)?;
                chunk.1 += buf.len();
                self.last_sizes.push(buf.len());
            }

            Ok(())
        }

        pub(super) fn pop_chunk(&mut self) -> Option<Vec<Vec<u8>>> {
            match self.chunks.pop_front() {
                Some((chunk, _)) if !chunk.is_empty() => Some(chunk),
                _ => None,
            }
        }
    }
}
