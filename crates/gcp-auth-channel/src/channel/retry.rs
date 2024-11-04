#![allow(dead_code)]

//! [Service] retry utilities for sessions/transactions
use std::future::Future;
use std::pin::Pin;
use std::sync::{mpsc, Arc};
use std::task::{Context, Poll};

use bytes::{Buf, Bytes};
use http::request::Parts;
use http_body::{Body, Frame};

pub mod on_status;

use tonic::client::GrpcService;

pub(super) struct RetryService<S, P> {
    policy: Arc<P>,
    service: S,
}

pub(crate) trait RetryPolicy<ReqBody: Body, S: GrpcService<CachedBody<ReqBody>>> {
    type Error: Into<crate::Error>;
    type PrepRetryFuture: Future<Output = Result<(), Self::Error>>;

    fn handle_response(
        &self,
        response: http::Response<S::ResponseBody>,
    ) -> Result<http::Response<S::ResponseBody>, Self::PrepRetryFuture>;

    fn handle_error(&self, error: &S::Error) -> Result<(), Self::PrepRetryFuture>;
}

impl<ReqBody, S, P> GrpcService<ReqBody> for RetryService<S, P>
where
    ReqBody: Body,
    S: GrpcService<CachedBody<ReqBody>> + Clone,
    P: RetryPolicy<ReqBody, S>,
    crate::Error: From<S::Error>,
{
    type Error = crate::Error;
    type ResponseBody = S::ResponseBody;
    type Future = RetryFuture<ReqBody, S, P>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        S::poll_ready(&mut self.service, cx).map_err(crate::Error::from)
    }

    fn call(&mut self, request: http::Request<ReqBody>) -> Self::Future {
        let (request_parts, body) = request.into_parts();

        let (sender, rx) = mpsc::channel();

        let size_hint = body.size_hint();

        let buf = Vec::with_capacity(size_hint.exact().unwrap_or(size_hint.lower()) as usize);

        let body = CachedBody::Original { body, sender };
        let request = http::Request::from_parts(request_parts.clone(), body);

        RetryFuture {
            request_parts,
            policy: Arc::clone(&self.policy),
            service: self.service.clone(),
            body: CachedBodyState::Recieving { rx, buf },
            last_error: None,
            state: RetryFutureState::Calling {
                future: self.service.call(request),
            },
        }
    }
}

pin_project_lite::pin_project! {
    pub struct RetryFuture<B: Body, S: GrpcService<CachedBody<B>>, P: RetryPolicy<B, S>> {
        request_parts: Parts,
        policy: Arc<P>,
        service: S,
        last_error: Option<S::Error>,
        body: CachedBodyState,
        #[pin]
        state: RetryFutureState<S::Future, P::PrepRetryFuture>,
    }
}

impl<B, S, P> Future for RetryFuture<B, S, P>
where
    B: Body,
    S: GrpcService<CachedBody<B>>,
    crate::Error: From<S::Error>,
    P: RetryPolicy<B, S>,
{
    type Output = crate::Result<http::Response<S::ResponseBody>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match this.state.as_mut().project() {
                RetryFutureStateProjection::Calling { future } => {
                    match std::task::ready!(future.poll(cx)) {
                        Ok(response) => match this.policy.handle_response(response) {
                            Ok(response) => return Poll::Ready(Ok(response)),
                            Err(retry) => this
                                .state
                                .set(RetryFutureState::PreppingRetry { future: retry }),
                        },
                        Err(err) => match this.policy.handle_error(&err) {
                            Ok(()) => return Poll::Ready(Err(crate::Error::from(err))),
                            Err(retry) => this
                                .state
                                .set(RetryFutureState::PreppingRetry { future: retry }),
                        },
                    }
                }
                RetryFutureStateProjection::PreppingRetry { future } => {
                    std::task::ready!(future.poll(cx)).map_err(Into::into)?;
                    this.state.set(RetryFutureState::PollReady);
                }
                RetryFutureStateProjection::PollReady => {
                    std::task::ready!(this.service.poll_ready(cx))?;

                    let parts = this.request_parts.clone();
                    this.body.poll();

                    let data = match this.body {
                        CachedBodyState::Recieved { frames } => Arc::clone(frames),
                        CachedBodyState::Recieving { .. } => panic!("invalid state"),
                    };

                    let body = CachedBody::Cached { data, next: 0 };

                    this.state.set(RetryFutureState::Calling {
                        future: this.service.call(http::Request::from_parts(parts, body)),
                    });
                }
            }
        }
    }
}

impl CachedBodyState {
    fn poll(&mut self) {
        if let Self::Recieving { rx, buf } = self {
            loop {
                match rx.try_recv() {
                    Ok(frame) => buf.push(frame),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        *self = Self::Recieved {
                            frames: std::mem::take(buf).into_boxed_slice().into(),
                        };
                        break;
                    }
                }
            }
        }
    }
}

enum CachedBodyState {
    Recieving {
        rx: mpsc::Receiver<Frame<Bytes>>,
        buf: Vec<Frame<Bytes>>,
    },
    Recieved {
        frames: Arc<[Frame<Bytes>]>,
    },
}

pin_project_lite::pin_project! {
    #[project = RetryFutureStateProjection]
    enum RetryFutureState<CallFut, PrepRetryFut> {
        Calling {
            #[pin]
            future: CallFut,
        },
        PreppingRetry {
            #[pin]
            future: PrepRetryFut,
        },
        PollReady,
    }
}

pin_project_lite::pin_project! {
    #[project = CachedBodyProjection]
    pub enum CachedBody<B: Body> {
        Cached {
            data: Arc<[Frame<Bytes>]>,
            next: usize,
        },
        Original {
            #[pin]
            body: B,
            sender: mpsc::Sender<Frame<Bytes>>,
        },
    }
}

fn clone_frame(frame: &Frame<Bytes>) -> Frame<Bytes> {
    if let Some(data) = frame.data_ref() {
        Frame::data(data.clone())
    } else if let Some(trailers) = frame.trailers_ref() {
        Frame::trailers(trailers.clone())
    } else {
        unreachable!("internally Frame is a 2 variant enum, this should be unreachable")
    }
}

impl<B: Body> Body for CachedBody<B> {
    type Data = Bytes;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.project() {
            CachedBodyProjection::Cached { data, next } => match data.get(*next) {
                Some(frame) => {
                    *next += 1;
                    Poll::Ready(Some(Ok(clone_frame(frame))))
                }
                None => Poll::Ready(None),
            },
            CachedBodyProjection::Original { body, sender } => {
                match std::task::ready!(body.poll_frame(cx)) {
                    Some(Ok(frame)) => {
                        let frame = frame.map_data(|mut data| data.copy_to_bytes(data.remaining()));

                        // if for whatever reason the reciever dies, we still
                        // want to let the original body complete, it
                        // might succeed on the first request and get around the issue
                        let _ = sender.send(clone_frame(&frame));
                        Poll::Ready(Some(Ok(frame)))
                    }
                    Some(Err(err)) => Poll::Ready(Some(Err(err))),
                    None => Poll::Ready(None),
                }
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Cached { data, next } => data.len() <= *next,
            Self::Original { body, .. } => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self {
            Self::Cached { data, next } => {
                http_body::SizeHint::with_exact((data.len() - next) as u64)
            }
            Self::Original { body, .. } => body.size_hint(),
        }
    }
}
