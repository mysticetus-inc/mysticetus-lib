use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{BufMut, Bytes, BytesMut};
use http_body::Body;
use hyper::body::Incoming;

pin_project_lite::pin_project! {
    #[project = CollectBodyProjection]
    pub(crate) struct CollectBody {
        #[pin]
        body: Option<Incoming>,
        state: State,
    }
}

pub(crate) fn collect_body(body: Incoming) -> CollectBody {
    CollectBody {
        body: Some(body),
        state: State::Pending,
    }
}

enum State {
    Pending,
    FirstChunk { buf: Bytes },
    CopyingRemaining { dst: BytesMut },
}

impl Future for CollectBody {
    type Output = Result<Bytes, hyper::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let maybe_chunk = std::task::ready!(this.poll_frame(cx))?;

            *this.state = match (&mut *this.state, maybe_chunk) {
                (State::Pending, None) => return Poll::Ready(Ok(Bytes::new())),
                (State::Pending, Some(buf)) => State::FirstChunk { buf },
                (State::FirstChunk { buf }, None) => return Poll::Ready(Ok(std::mem::take(buf))),
                (State::FirstChunk { buf }, Some(second_chunk)) => {
                    let first_chunk = std::mem::take(buf);
                    let mut dst = match first_chunk.try_into_mut() {
                        Ok(dst) => dst,
                        Err(bytes) => {
                            let mut dst = BytesMut::with_capacity(bytes.len() + second_chunk.len());
                            dst.put(bytes);
                            dst
                        }
                    };

                    dst.put(second_chunk);

                    State::CopyingRemaining { dst }
                }
                (State::CopyingRemaining { dst }, Some(chunk)) => {
                    dst.put(chunk);
                    continue;
                }
                (State::CopyingRemaining { dst }, None) => {
                    let buf = std::mem::take(dst).freeze();
                    return Poll::Ready(Ok(buf));
                }
            }
        }
    }
}

impl CollectBodyProjection<'_> {
    fn poll_frame(&mut self, cx: &mut Context<'_>) -> Poll<Result<Option<Bytes>, hyper::Error>> {
        let Some(mut body) = self.body.as_mut().as_pin_mut() else {
            return Poll::Ready(Ok(None));
        };

        loop {
            let frame = match std::task::ready!(body.as_mut().poll_frame(cx)) {
                Some(frame) => frame?,
                None => {
                    self.body.set(None);
                    return Poll::Ready(Ok(None));
                }
            };

            if let Ok(data) = frame.into_data() {
                return Poll::Ready(Ok(Some(data)));
            }
        }
    }
}
