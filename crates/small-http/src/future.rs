use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::body::Incoming;

use crate::{ConnectionParts, HttpVersion, RawRequestFuture, SendRequest};

pin_project_lite::pin_project! {
    pub struct RequestFuture<'a, B, HttpVer: HttpVersion<B>> {
        parts: &'a mut ConnectionParts<HttpVer::SendRequest>,
        #[pin]
        state: State<B>,
    }
}

impl<'a, B, HttpVer: HttpVersion<B>> RequestFuture<'a, B, HttpVer> {
    pub(crate) fn new(
        parts: &'a mut ConnectionParts<HttpVer::SendRequest>,
        request: http::Request<B>,
    ) -> Self {
        Self {
            parts,
            state: State::PollReady {
                request: Some(request),
            },
        }
    }
}

pin_project_lite::pin_project! {
    #[project = StateProj]
    enum State<B> {
        PollReady {
            request: Option<http::Request<B>>,
        },
        Calling,
    }
}

impl<B, HttpVer> Future for RequestFuture<'_, B, HttpVer>
where
    HttpVer: HttpVersion<B>,
    B: 'static,
{
    type Output = Result<http::Response<Incoming>, crate::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match this.state.as_mut().project() {
                StateProj::Calling => {
                    let fut = this.request_fut.as_mut().expect("invalid state");
                    let result = std::task::ready!(fut.poll(cx));
                    return Poll::Ready(result.map_err(crate::Error::from));
                }
                StateProj::PollReady { request } => {
                    let poll_ready = self.parts.
                    let (send_request, _) = parts.as_mut().expect("invalid state");
                    std::task::ready!(send_request.poll_ready(cx))?;

                    let (send_request, request) = parts.take().expect("invalid state");

                    match this.request_fut {
                        None => {
                            **this.request_fut =
                                Some(RawRequestFuture::new(send_request.send_request(request)))
                        }
                        Some(fut) => {
                            fut.set(send_request.send_request(request));
                        }
                    }
                }
            }
        }
    }
}
