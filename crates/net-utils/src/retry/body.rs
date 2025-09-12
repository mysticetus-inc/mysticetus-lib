//! Implementation loosely based on the `linkerd2-proxy` [`http-retry`] crate.
//!
//! [`http-retry`]: <https://github.com/linkerd/linkerd2-proxy/blob/1cff3aef82c203bf09ccce485506d7a29ca27308/linkerd/http-retry>
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::task::{Context, Poll};

use bytes::{Buf, Bytes};
use http_body::{Body, Frame, SizeHint};
use parking_lot::Mutex;

use crate::retry::body::buf_list::BufList;
use crate::retry::body::buffered::BufferedBody;

mod buf_list;
mod buffered;

pub fn wrap_body<B: UnpinBody>(body: B) -> (RetryHandle<B>, RetryableBody<B>) {
    let (frame_buffer_sender, frame_buffer_recv) = mpsc::channel();

    let shared = Arc::new(SharedState {
        errored: AtomicBool::new(false),
        frame_buffer_sender,
        state: Mutex::new(BodyState {
            remaining_body: None,
            frame_buffer_recv,
            buffered: BufList::default(),
            trailers: None,
        }),
    });

    let handle = RetryHandle { shared };

    let wrapped = RetryableBody {
        body: WrappedBody::Raw { body },
        shared: Arc::clone(&handle.shared),
    };

    (handle, wrapped)
}

// trait alias to get around pin_project_lite macro parsing issues
pub trait UnpinBody: Unpin + Body {}

impl<B: Body + Unpin> UnpinBody for B {}

pin_project_lite::pin_project! {
    pub struct RetryableBody<B: UnpinBody> {
        body: WrappedBody<B>,
        shared: Arc<SharedState<B>>,
    }

    impl<B: UnpinBody> PinnedDrop for RetryableBody<B> {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();

            // if the original body isn't done, we need to pass it back to the shared
            // state so we can still get access to it during a retry attempt

            if let Some(body) = this.body.take_remaining_body() {
                this.shared.state.lock().remaining_body = Some(body);
            }
        }
    }
}

impl<B: UnpinBody> Body for RetryableBody<B> {
    type Data = Bytes;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        this.body.poll_frame(cx, &**this.shared)
    }

    fn is_end_stream(&self) -> bool {
        match self.body {
            WrappedBody::Empty => true,
            WrappedBody::Raw { ref body } => body.is_end_stream(),
            WrappedBody::Buffered { ref body } => body.is_end_stream(),
            WrappedBody::PartiallyBuffered {
                ref body,
                ref buffered,
            } => !buffered.has_remaining() && body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}

pin_project_lite::pin_project! {
    #[project = WrappedBodyProjection]
    enum WrappedBody<B> {
        Empty,
        Raw { body: B },
        PartiallyBuffered {
            buffered: BufList,
            body: B,
        },
        Buffered { body: BufferedBody },
    }
}

impl<B: Body> WrappedBody<B> {
    fn take_remaining_body(&mut self) -> Option<B> {
        match std::mem::replace(self, Self::Empty) {
            Self::PartiallyBuffered { body, .. } | Self::Raw { body } if !body.is_end_stream() => {
                Some(body)
            }
            _ => None,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty => SizeHint::with_exact(0),
            Self::Raw { body } => body.size_hint(),
            Self::Buffered { body } => body.size_hint(),
            Self::PartiallyBuffered { buffered, body } => {
                let len = buffered.remaining() as u64;
                let body_hint = body.size_hint();

                if len == 0 {
                    return body_hint;
                }

                let mut ret = SizeHint::new();
                ret.set_lower(len + body_hint.lower());

                if let Some(body_upper) = body_hint.upper() {
                    ret.set_upper(len + body_upper);
                }

                ret
            }
        }
    }

    fn poll_frame(
        &mut self,
        cx: &mut Context<'_>,
        shared: &SharedState<B>,
    ) -> Poll<Option<Result<Frame<Bytes>, B::Error>>>
    where
        B: Unpin,
    {
        fn convert_and_clone_frame<B: Buf>(
            frame: Frame<B>,
        ) -> (Frame<Bytes>, Option<Frame<Bytes>>) {
            let frame = frame.map_data(|mut data| data.copy_to_bytes(data.remaining()));

            let cloned = if let Some(data) = frame.data_ref() {
                Some(Frame::data(data.clone()))
            } else if let Some(trailers) = frame.trailers_ref() {
                Some(Frame::trailers(trailers.clone()))
            } else {
                None
            };

            (frame, cloned)
        }

        let body = match self {
            Self::Empty => return Poll::Ready(None),
            Self::Buffered { body } => match body.pop_frame() {
                Some(frame) => return Poll::Ready(Some(Ok(frame))),
                None => {
                    *self = Self::Empty;
                    return Poll::Ready(None);
                }
            },
            Self::Raw { body } => body,
            Self::PartiallyBuffered { buffered, body } => {
                if let Some(chunk) = buffered.pop() {
                    return Poll::Ready(Some(Ok(Frame::data(chunk))));
                }

                body
            }
        };

        match std::task::ready!(Pin::new(body).poll_frame(cx)) {
            Some(Ok(frame)) => {
                let (frame, cloned) = convert_and_clone_frame(frame);

                if let Some(cloned) = cloned {
                    shared
                        .frame_buffer_sender
                        .send(cloned)
                        .expect("we never drop the receiver");
                }

                Poll::Ready(Some(Ok(frame)))
            }
            Some(Err(err)) => {
                shared.errored.store(true, Ordering::SeqCst);
                Poll::Ready(Some(Err(err)))
            }
            None => {
                *self = Self::Empty;
                Poll::Ready(None)
            }
        }
    }
}

pub struct RetryHandle<B> {
    shared: Arc<SharedState<B>>,
}

impl<B: UnpinBody> RetryHandle<B> {
    pub fn retry(&self) -> Option<RetryableBody<B>> {
        // if the wrapped body itself threw an error, we really
        // can't retry anything without risking data loss.
        if self.shared.errored.load(Ordering::SeqCst) {
            return None;
        }

        let mut guard = self.shared.state.lock();
        guard.collect_buffered();

        let remaining_body = guard.remaining_body.take();
        let buffered = guard.buffered.clone();
        let trailers = guard.trailers.clone();
        drop(guard);

        let body = match remaining_body {
            Some(body) if buffered.has_remaining() => {
                WrappedBody::PartiallyBuffered { buffered, body }
            }
            Some(body) => WrappedBody::Raw { body },
            None if buffered.has_remaining() => WrappedBody::Buffered {
                body: BufferedBody::new(buffered, trailers),
            },
            None => WrappedBody::Empty,
        };

        Some(RetryableBody {
            body,
            shared: Arc::clone(&self.shared),
        })
    }
}

struct SharedState<B> {
    errored: AtomicBool,
    frame_buffer_sender: mpsc::Sender<Frame<Bytes>>,
    state: Mutex<BodyState<B>>,
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    struct BodyState<B> {
        remaining_body: Option<B>,
        frame_buffer_recv: mpsc::Receiver<Frame<Bytes>>,
        buffered: buf_list::BufList,
        trailers: Option<http::HeaderMap>,
    }
}

impl<B> BodyState<B> {
    fn collect_buffered(&mut self) {
        for frame in self.frame_buffer_recv.try_iter() {
            match frame.into_data() {
                Ok(data) => self.buffered.push(data),
                Err(frame) => match frame.into_trailers() {
                    Ok(trailers) => match self.trailers {
                        Some(ref mut existing) => existing.extend(trailers),
                        None => self.trailers = Some(trailers),
                    },
                    // there shouldn't be any other frame types, but if another
                    // type gets added, we should just ignore it (but panic in debug mode)
                    Err(frame) => debug_assert!(false, "got an unexpected frame type: {frame:#?}"),
                },
            }
        }
    }
}
