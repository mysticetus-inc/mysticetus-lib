use std::pin::Pin;
use std::task::{Context, Poll};

use http::request::Parts;

use crate::backoff::{Backoff, BackoffConfig};
use crate::http_svc::HttpResponse;
use crate::retry::body::{RetryHandle, RetryableBody, UnpinBody};
use crate::retry::classify::{ClassifyResponse, ShouldRetry};

pin_project_lite::pin_project! {
    pub struct RetryFuture<
        ReqBody: UnpinBody,
        Svc: tower::Service<http::Request<RetryableBody<ReqBody>>>,
        Classify: ClassifyResponse<Svc::Response, Svc::Error>,
    > {
        svc: Svc,
        classify: Classify,
        request_parts: Parts,
        body_handle: RetryHandle<ReqBody>,
        backoff: Backoff,
        // Only Some when the state is PollReady or BackingOff
        next: Option<RetryableBody<ReqBody>>,
        #[pin]
        state: State<Svc::Future>,
    }
}

impl<ReqBody, Svc, Classify> RetryFuture<ReqBody, Svc, Classify>
where
    ReqBody: UnpinBody,
    Svc: tower::Service<http::Request<RetryableBody<ReqBody>>>,
    Classify: ClassifyResponse<Svc::Response, Svc::Error>,
{
    pub(super) fn new(
        mut svc: Svc,
        request: http::Request<ReqBody>,
        classify: Classify,
        backoff_config: &BackoffConfig,
    ) -> Self {
        let (request_parts, body) = request.into_parts();
        let (body_handle, retry_body) = super::body::wrap_body(body);
        let request = http::Request::from_parts(request_parts.clone(), retry_body);
        let fut = svc.call(request);

        Self {
            svc,
            classify,
            body_handle,
            request_parts,
            next: None,
            backoff: backoff_config.make_backoff(),
            state: State::Calling { fut },
        }
    }
}

pin_project_lite::pin_project! {
    #[project = StateProjection]
    enum State<F> {
        PollReady,
        Calling { #[pin] fut: F },
        BackingOff { #[pin] sleep: tokio::time::Sleep },
    }
}

impl<ReqBody, Svc, Classify> Future for RetryFuture<ReqBody, Svc, Classify>
where
    ReqBody: UnpinBody,
    Svc: tower::Service<http::Request<RetryableBody<ReqBody>>>,
    Svc::Response: HttpResponse,
    Classify: ClassifyResponse<Svc::Response, Svc::Error>,
{
    type Output = Result<Svc::Response, Svc::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use StateProjection::*;

        let mut this = self.project();

        loop {
            match this.state.as_mut().project() {
                PollReady => {
                    std::task::ready!(this.svc.poll_ready(cx))?;

                    this.state.as_mut().set(State::Calling {
                        fut: this.svc.call(http::Request::from_parts(
                            this.request_parts.clone(),
                            this.next.take().expect("invalid state"),
                        )),
                    });
                }
                BackingOff { sleep } => {
                    std::task::ready!(sleep.poll(cx));
                    this.state.as_mut().set(State::PollReady);
                }
                Calling { fut } => {
                    let result = std::task::ready!(fut.poll(cx));

                    if matches!(this.classify.should_retry(&result), ShouldRetry::Yes) {
                        return Poll::Ready(result);
                    }

                    match this.backoff.backoff_once() {
                        Some(backoff) => this.state.as_mut().set(State::BackingOff {
                            sleep: backoff.into_future(),
                        }),
                        // exhausted all retries, return the response as is
                        None => return Poll::Ready(result),
                    }
                }
            }
        }
    }
}
