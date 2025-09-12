use std::sync::Arc;

use gcp_auth_channel::channel::headers::{Http, WithHeaders};
use gcp_auth_channel::{AuthChannel, Scope};
use protos::bigquery_storage::big_query_write_client::BigQueryWriteClient;
use protos::bigquery_storage::write_stream::Type;
use protos::bigquery_storage::{BatchCommitWriteStreamsRequest, FinalizeWriteStreamResponse};
use tonic::transport::Channel;
use typenum::marker_traits::Bit;
use typenum::{B0, B1};

use super::{BigQueryStorageClient, Error, WriteSession, WriteSessionInner};

mod private {
    pub trait Sealed {}
}

/// Private, sealed trait representing a write stream type.
pub trait WriteStreamType: private::Sealed {
    /// Marker type of whether a stream can/should call the FinalizeWriteStream endpoint.
    type CanFinalize: Bit;
    /// Marker type of whether a stream can call the BatchCommitWriteStreams endpoint.
    type CanBatchCommit: Bit;
    /// Marker type of whether a stream can call the FlushRows endpoint.
    type CanFlush: Bit;

    fn to_type() -> Type;
}

/// private, sealed trait for streams that can be finalized, allowing for custom
/// behavior.
pub trait FinalizeStream: WriteStreamType<CanFinalize = B1> + Sized {
    type Ok;

    /// Called after the finalize response has been successfully recieved.
    /// Depending on the stream, there may be extra work that needs to be done
    fn on_finalized<R>(
        session: WriteSession<Self, R>,
        resp: FinalizeWriteStreamResponse,
    ) -> Result<Self::Ok, Error>;
}

macro_rules! impl_stream_types {
    ($stream_type:ident { $type_variant:ident } ($finalize:ty, $commit:ty, $flush:ty)) => {
        #[doc = "Marker Type specifying a "]
        #[doc = stringify!($stream_type)]
        #[doc = " write stream."]
        #[doc = ""]
        #[doc = "The different stream types are described in more detail here:"]
        #[doc = ""]
        #[doc = "<https://cloud.google.com/bigquery/docs/write-api#application-created_streams>"]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $stream_type;

        impl private::Sealed for $stream_type { }

        impl WriteStreamType for $stream_type {
            type CanFinalize = $finalize;
            type CanBatchCommit = $commit;
            type CanFlush = $flush;

            fn to_type() -> Type {
                Type::$type_variant
            }
        }
    };
    ($stream_type:ident( $finalize:ty, $commit:ty, $flush:ty )) => {
        impl_stream_types!($stream_type { $stream_type } ($finalize, $commit, $flush));
    };
    (
        $(
            $stream_type:ident
            $({ $type_variant:ident })?
            ( $finalize:ty, $commit:ty, $flush:ty )
        ),*
        $(,)?
    ) => {
        $(
            impl_stream_types!($stream_type $({ $type_variant })? ($finalize, $commit, $flush));
        )*
    };
}

macro_rules! impl_noop_finalize {
    ($($stream:ident),* $(,)?) => {
        $(
            impl FinalizeStream for $stream {
                type Ok = ();

                fn on_finalized<R>(
                    _: WriteSession<Self, R>,
                    _: FinalizeWriteStreamResponse,
                ) -> Result<Self::Ok, Error> {
                    Ok(())
                }
            }
        )*
    };
}

impl_stream_types! {
    Buffered(B1, B0, B1),
    Committed(B1, B0, B0),
    Default { Committed } (B0, B0, B0),
    Pending(B1, B1, B0),
}

impl_noop_finalize!(Buffered, Committed);

impl FinalizeStream for Pending {
    type Ok = PendingStream;

    fn on_finalized<R>(
        session: WriteSession<Self, R>,
        resp: FinalizeWriteStreamResponse,
    ) -> Result<Self::Ok, Error> {
        Ok(PendingStream {
            inner: session.inner,
            rows: resp.row_count as usize,
            channel: session.channel,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PendingStream {
    inner: Arc<WriteSessionInner>,
    rows: usize,
    channel: AuthChannel<WithHeaders<Channel, Http>>,
}

impl PendingStream {
    pub async fn commit(self) -> Result<usize, Error> {
        self.commit_many(std::iter::empty()).await
    }

    pub async fn commit_many<I>(self, others: I) -> Result<usize, Error>
    where
        I: IntoIterator<Item = Self>,
    {
        let iter = others.into_iter();
        let (low, high) = iter.size_hint();

        let mut write_streams = Vec::with_capacity(1 + high.unwrap_or(low));

        let parent_end_idx = self.inner.write_stream.name.find("/streams/").unwrap();

        let parent = self.inner.write_stream.name.as_str()[..parent_end_idx].to_owned();

        macro_rules! unwrap_name {
            ($parent:expr) => {{
                match Arc::try_unwrap($parent.inner) {
                    Ok(ws) => ws.write_stream.name,
                    Err(arc) => arc.write_stream.name.clone(),
                }
            }};
        }

        write_streams.push(unwrap_name!(self));
        let mut rows = self.rows;

        for pending in iter {
            rows += pending.rows;
            write_streams.push(unwrap_name!(pending));
        }

        let req = BatchCommitWriteStreamsRequest {
            parent,
            write_streams,
        };

        let mut client = BigQueryWriteClient::new(self.channel);

        let resp = client.batch_commit_write_streams(req).await?.into_inner();

        crate::error::CommitError::from_raw_errors(resp.stream_errors)?;

        Ok(rows)
    }
}
