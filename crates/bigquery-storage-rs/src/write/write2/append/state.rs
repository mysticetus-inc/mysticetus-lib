use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use net_utils::bidi2::RequestSink;
use protos::bigquery_storage::{AppendRowsRequest, AppendRowsResponse};

use super::{Schema, StreamType, WriteSession};

pin_project_lite::pin_project! {
    pub struct AppendRowsState {
        pending_responses: usize,
        request_state: RequestState,
        options: AppendOptions,
        sink: RequestSink<AppendRowsRequest>,
        #[pin]
        stream: tonic::Streaming<AppendRowsResponse>,
    }
}

impl AppendRowsState {
    pub(super) fn send(&mut self, request: AppendRowsRequest) -> crate::Result<()> {
        self.sink.send(request).map_err(|_| {
            crate::Error::Status(tonic::Status::aborted("client request streaming closed"))
        })?;

        self.pending_responses += 1;

        Ok(())
    }

    pub(super) fn close_sink(&mut self) {
        self.sink.close();
    }

    pub(super) fn pending_responses(&self) -> usize {
        self.pending_responses
    }

    pub(super) fn request_state(&self) -> RequestState {
        self.request_state
    }

    pub(super) fn poll_next_message<R, Type: StreamType>(
        &mut self,
        session: &WriteSession<R, Type>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<NewOffset>>> {
        let Some(result) = std::task::ready!(Pin::new(&mut self.stream).poll_next(cx)) else {
            return Poll::Ready(Ok(None));
        };

        let mut response = result?;

        // TODO: make saturating w/ logging on the underflow case?
        self.pending_responses = self
            .pending_responses
            .checked_sub(1)
            .expect("got more responses than expected");

        if let Some(schema) = response.updated_schema.take() {
            let schema = Schema::from_table_schema(schema)?;
            session.update_schema(schema);
            self.request_state.remove(RequestState::SCHEMA_CHANGE);
        }

        match crate::error::RowInsertErrors::from_raw_response(response)? {
            Some(offset) => Poll::Ready(Ok(Some(NewOffset::Offset(offset)))),
            None => Poll::Ready(Ok(Some(NewOffset::Unknown))),
        }
    }

    pub(super) async fn next_message<R, Type: StreamType>(
        &mut self,
        session: &WriteSession<R, Type>,
    ) -> crate::Result<Option<NewOffset>> {
        std::future::poll_fn(|cx| self.poll_next_message(session, cx)).await
    }
}

pub enum NewOffset {
    Offset(i64),
    Unknown,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct RequestState: u8 {
        const FIRST = 1;
        const TARGET_CHANGED = 2;
        const SCHEMA_CHANGE = 4;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidateMessageCount {
    Dont,
    #[default]
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AppendOptions {
    pub validate_message_count: ValidateMessageCount,
}

impl RequestState {
    pub(crate) fn needs_name(&self) -> bool {
        self.contains(Self::FIRST) || self.contains(Self::TARGET_CHANGED)
    }

    pub(crate) fn needs_schema(&self) -> bool {
        self.contains(Self::FIRST) || self.contains(Self::SCHEMA_CHANGE)
    }
}
