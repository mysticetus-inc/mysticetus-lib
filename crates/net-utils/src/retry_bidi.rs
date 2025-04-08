use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio_util::sync::ReusableBoxFuture;

use crate::backoff::{Backoff, BackoffOnce};
use crate::bidi2::{RequestSink, RequestStream};
use crate::transient::{DefaultTransientErrors, IsTransient};

pin_project_lite::pin_project! {
    pub struct RetryBidi<
        Method: BidiMethod,
        Client: GrpcClient<Method>,
        Classify: IsTransient = DefaultTransientErrors,
    > {
        parts: RetryParts<Method, Client, Classify>,
        #[pin]
        state: State<Method>,
    }
}

impl<Method: BidiMethod, Client: GrpcClient<Method>> RetryBidi<Method, Client> {
    pub fn new(client: Client, init_req: Option<Method::Request>) -> Self {
        Self::new_opt(
            client,
            init_req,
            DefaultTransientErrors,
            false,
            Backoff::default(),
        )
    }
}

impl<Method: BidiMethod, Client: GrpcClient<Method>, Classify: IsTransient>
    RetryBidi<Method, Client, Classify>
{
    pub fn new_opt(
        client: Client,
        init_req: Option<Method::Request>,
        classify: Classify,
        retry_on_close: bool,
        backoff: Backoff,
    ) -> Self {
        Self {
            parts: RetryParts {
                closing: false,
                retry_on_close,
                init_stream_fut: ReusableBoxFuture::new(init_stream(
                    client.clone(),
                    init_req.clone(),
                )),
                backoff,
                client,
                pending: VecDeque::new(),
                init_req,
                request_state: Default::default(),
                classify,
            },
            state: State::Initializing { error: None },
        }
    }

    pub fn pending_requests(&self) -> usize {
        self.parts.pending.len()
    }

    pub fn send(&mut self, message: Method::Request) -> Result<(), Method::Request> {
        match self.state {
            State::Closed { .. } => Err(message),
            State::Streaming { ref mut sink, .. } => {
                // since message order matters, we can only send this if there's
                // no prior pending requests. therefore, try to send any pending
                // requests, that way we can try to send the new request right
                // away without buffering.
                self.parts.send_requests(sink);

                if self.parts.pending.is_empty() {
                    if let Err(error) = sink.send(message) {
                        self.parts.pending.push_back(error.0);
                    }
                } else {
                    self.parts.pending.push_back(message);
                }
                Ok(())
            }
            _ => {
                self.parts.pending.push_back(message);
                Ok(())
            }
        }
    }

    pub fn next(
        mut self: Pin<&mut Self>,
    ) -> std::future::PollFn<
        impl FnMut(&mut Context<'_>) -> Poll<Option<tonic::Result<Method::Response>>>,
    > {
        std::future::poll_fn(move |cx| self.as_mut().poll_next(cx))
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, State::Closed { .. })
    }

    /// Indicate that the stream should close gracefully (after sending any pending messages)
    pub fn close(&mut self) {
        self.parts.closing = true;
    }
}

impl<Method: BidiMethod, Client: GrpcClient<Method>, Classify: IsTransient> Stream
    for RetryBidi<Method, Client, Classify>
{
    type Item = tonic::Result<Method::Response>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let parts = this.parts;

        loop {
            match this.state.as_mut().project() {
                StateProjection::Closed { error } => match error.take() {
                    Some(error) => return Poll::Ready(Some(Err(error))),
                    None => return Poll::Ready(None),
                },
                StateProjection::Initializing { .. } => {
                    match std::task::ready!(parts.init_stream_fut.poll(cx)) {
                        Ok((sink, streaming)) => {
                            this.state.set(State::Streaming { sink, streaming })
                        }
                        Err(error) if parts.classify.is_transient(&error) => {
                            match parts.backoff.backoff_once() {
                                Some(backoff) => this.state.set(State::BackingOff {
                                    backoff: backoff.into_future(),
                                    error: Some(error),
                                }),
                                None => this.state.set(State::Closed { error: Some(error) }),
                            }
                        }
                        Err(error) => this.state.set(State::Closed { error: Some(error) }),
                    }
                }
                StateProjection::BackingOff { backoff, error } => {
                    std::task::ready!(backoff.poll(cx));
                    let client = parts.client.clone();
                    let error = error.take();
                    let init_req = parts.build_init_request();
                    parts.init_stream_fut.set(init_stream(client, init_req));
                    this.state.set(State::Initializing { error });
                }
                StateProjection::Streaming { sink, streaming } => {
                    match std::task::ready!(parts.poll_drive(sink, streaming, cx)) {
                        Ok(Ok(resp)) => return Poll::Ready(Some(Ok(resp))),
                        Ok(Err(error)) if parts.classify.is_transient(&error) => {
                            match parts.backoff.backoff_once() {
                                Some(backoff) => this.state.set(State::BackingOff {
                                    backoff: backoff.into_future(),
                                    error: Some(error),
                                }),
                                None => this.state.set(State::Closed { error: Some(error) }),
                            }
                        }
                        Ok(Err(error)) => this.state.set(State::Closed { error: Some(error) }),
                        Err(new_state) => this.state.set(new_state),
                    }
                }
            }
        }
    }
}

struct RetryParts<
    Method: BidiMethod,
    Client: GrpcClient<Method>,
    Classify: IsTransient = DefaultTransientErrors,
> {
    closing: bool,
    retry_on_close: bool,
    client: Client,
    backoff: Backoff,
    pending: VecDeque<Method::Request>,
    init_req: Option<Method::Request>,
    request_state: Method::State,
    init_stream_fut: ReusableBoxFuture<
        'static,
        tonic::Result<(
            RequestSink<Method::Request>,
            tonic::Streaming<Method::Response>,
        )>,
    >,
    classify: Classify,
}

impl<Method: BidiMethod, Client: GrpcClient<Method>, Classify: IsTransient>
    RetryParts<Method, Client, Classify>
{
    fn send_requests(&mut self, sink: &mut RequestSink<Method::Request>) {
        // feed as many requests to the stream as we can
        while let Some(next_req) = self.pending.pop_front() {
            if let Err(error) = sink.send(next_req) {
                self.pending.push_front(error.0);
                break;
            }
        }
    }

    fn build_init_request(&mut self) -> Option<Method::Request> {
        let state = std::mem::take(&mut self.request_state);
        Method::request_from_state(&self.init_req, state)
    }

    fn poll_drive(
        &mut self,
        sink: &mut RequestSink<Method::Request>,
        streaming: &mut tonic::Streaming<Method::Response>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<tonic::Result<Method::Response>, State<Method>>> {
        self.send_requests(sink);

        match std::task::ready!(Pin::new(streaming).poll_next(cx)) {
            // caller handles errors
            Some(Err(error)) => Poll::Ready(Ok(Err(error))),
            Some(Ok(mut resp)) => {
                // reset the backoff if we get a successful message
                self.backoff.reset();
                self.request_state = Method::extract_state(&mut resp);
                Poll::Ready(Ok(Ok(resp)))
            }
            None if (self.closing || !self.retry_on_close) && self.pending.is_empty() => {
                Poll::Ready(Err(State::Closed { error: None }))
            }
            // if we arent closing, or we are and have pending requests, restart the stream.
            // dont bother with retries here, since the server didnt return any errors.
            None => {
                let init_req = self.build_init_request();
                let client = self.client.clone();
                self.init_stream_fut.set(init_stream(client, init_req));
                Poll::Ready(Err(State::Initializing { error: None }))
            }
        }
    }
}

pin_project_lite::pin_project! {
    #[project = StateProjection]
    enum State<Method: BidiMethod> {
        Streaming {
            sink: RequestSink<Method::Request>,
            streaming: tonic::Streaming<Method::Response>,
        },
        BackingOff {
            #[pin]
            backoff: <BackoffOnce as IntoFuture>::IntoFuture,
            error: Option<tonic::Status>,
        },
        Initializing {
            error: Option<tonic::Status>,
        },
        Closed { error: Option<tonic::Status> }
    }
}

pub trait BidiMethod: 'static {
    type Request: std::fmt::Debug + Send + Clone + 'static;
    type Response: 'static;

    type State: Default;

    fn extract_state(response: &mut Self::Response) -> Self::State;

    fn request_from_state(
        init_request: &Option<Self::Request>,
        state: Self::State,
    ) -> Option<Self::Request>;
}

pub trait GrpcClient<Method: BidiMethod>: Send + Clone + 'static {
    // no Send + 'static bound, since we want to be maximally pessimistic about the signature of
    // generated tonic client methods.
    fn call(
        &mut self,
        req_stream: RequestStream<Method::Request>,
    ) -> impl Future<Output = tonic::Result<tonic::Streaming<Method::Response>>> + Send;
}

// use a single method for making the grpc call, that way the returned/desugared future
// should always be the same size for reuse in ReusableBoxFuture
async fn init_stream<Method: BidiMethod, Client: GrpcClient<Method>>(
    mut client: Client,
    init_req: Option<Method::Request>,
) -> tonic::Result<(
    RequestSink<Method::Request>,
    tonic::Streaming<Method::Response>,
)> {
    let (sink, stream) = crate::bidi2::build_pair();
    let streaming = client.call(stream).await?;

    if let Some(req) = init_req {
        if let Err(send_error) = sink.send(req) {
            return Err(tonic::Status::internal(format!(
                "stream closed immediately w/ request: {:#?}",
                send_error.0
            )));
        }
    }

    Ok((sink, streaming))
}
