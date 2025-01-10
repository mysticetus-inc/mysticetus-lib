use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;

use super::StreamType;
use super::state::NewOffset;

pin_project_lite::pin_project! {
    pub struct AppendStreamedRows<'session, S: Stream, Type: StreamType>
    where
        S::Item: IntoIterator<Item: serde::Serialize>,
    {
        #[pin]
        pub(super) row_stream: Option<S>,
        pub(super) encoder: super::RowEncoder<<S::Item as IntoIterator>::Item>,
        pub(super) ctx: super::AppendRowsContext<'session, <S::Item as IntoIterator>::Item, Type>,
    }
}

impl<S, Type> Future for AppendStreamedRows<'_, S, Type>
where
    S: Stream<Item: IntoIterator<Item: serde::Serialize>>,
    Type: StreamType,
{
    type Output = crate::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if let Some(row_stream) = this.row_stream.as_mut().as_pin_mut() {
            match row_stream.poll_next(cx) {
                Poll::Pending if this.ctx.state.pending_responses() == 0 => return Poll::Pending,
                Poll::Pending => (),
                Poll::Ready(None) => {
                    this.row_stream.set(None);
                    let mut guard = None;
                    if let Some(last_request) = this.encoder.take_if_not_empty(
                        &this.ctx.session,
                        &mut guard,
                        this.ctx.state.request_state(),
                    ) {
                        this.ctx.state.send(last_request)?;
                    }

                    this.ctx.state.close_sink();
                }
                Poll::Ready(Some(rows)) => {
                    let state = this.ctx.session.state();
                    for row in rows {
                        match this.encoder.append_row(
                            &this.ctx.session,
                            state.schema(),
                            this.ctx.state.request_state(),
                            row,
                        ) {
                            Ok(None) => (),
                            Ok(Some(request)) => {
                                this.ctx.state.send(request)?;
                            }
                            Err(error) => return Poll::Ready(Err(error)),
                        }
                    }
                }
            }
        }

        loop {
            match std::task::ready!(this.ctx.poll_next_message(cx))? {
                Some(NewOffset::Unknown) => (),
                Some(NewOffset::Offset(offset)) => this.ctx.session.update_offset(offset),
                None => {
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}
