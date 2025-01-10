use std::task::{Context, Poll};

use encoder::RowEncoder;
use futures::Stream;
use protos::bigquery_storage::AppendRowsRequest;
use state::{AppendRowsState, NewOffset};

use crate::write::Schema;
use crate::write::write2::types::Boolean;

mod encoder;
mod state;
mod stream;

pub(super) use state::RequestState;

use super::maybe_owned_mut::MaybeOwnedMut;
use super::{DefaultStream, StreamType, WriteSession};

pin_project_lite::pin_project! {
    #[project = AppendRowsContextProjection]
    pub struct AppendRowsContext<'a, R, Type: StreamType = DefaultStream> {
        session: MaybeOwnedMut<'a, WriteSession<R, Type>>,
        #[pin]
        state: AppendRowsState,
    }
}

pub struct EncodedRequest(AppendRowsRequest);

impl<'a, R, Type> AppendRowsContext<'a, R, Type>
where
    R: serde::Serialize,
    Type: StreamType,
{
    async fn send_one(&mut self, request: AppendRowsRequest) -> crate::Result<NewOffset> {
        self.state.send(request)?;

        self.state
            .next_message(&self.session)
            .await?
            .ok_or_else(|| crate::Error::Status(tonic::Status::aborted("server streaming closed")))
    }

    pub fn append_streamed<S>(self, row_stream: S) -> stream::AppendStreamedRows<'a, S, Type>
    where
        S: Stream<Item: IntoIterator<Item = R>>,
    {
        let encoder = RowEncoder::new(
            &*self.session,
            self.session.make_base_message(self.state.request_state()),
        );

        stream::AppendStreamedRows {
            row_stream: Some(row_stream),
            encoder,
            ctx: self,
        }
    }

    fn poll_next_message(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<NewOffset>>> {
        self.state.poll_next_message(&self.session, cx)
    }

    pub async fn append(mut self, rows: impl IntoIterator<Item = R>) -> crate::Result<()> {
        let mut state = None;

        let init_msg = crate::write::write2::session::make_base_message(
            self.session.shared(),
            &mut state,
            self.state.request_state(),
        );

        let mut encoder = RowEncoder::new(&self.session, init_msg);

        let mut row_iter = rows.into_iter();

        let session_state = state.get_or_insert_with(|| self.session.state());

        while let Some(row) = row_iter.next() {
            if let Some(request) = encoder.append_row(
                &self.session,
                session_state.schema(),
                self.state.request_state(),
                row,
            )? {
                self.state.send(request)?;
            }
        }

        if let Some(request) =
            encoder.take_if_not_empty(&self.session, &mut state, self.state.request_state())
        {
            self.state.send(request)?;
        }

        self.state.close_sink();
        // drop(state);

        loop {
            let offset = match self.state.next_message(&self.session).await? {
                None => return Ok(()),
                Some(offset) => offset,
            };

            if <Type::OffsetAllowed as Boolean>::VALUE {
                if let NewOffset::Offset(offset) = offset {
                    self.session
                        .state_mut()
                        .stream_type_mut()
                        .update_offset(offset);
                }
            }
        }
    }
}
