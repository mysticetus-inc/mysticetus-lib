use std::borrow::Cow;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use hyper::body::Incoming;
use hyper_util::client::legacy::{Client as HyperClient, ResponseFuture as HyperResponseFuture};
use net_utils::backoff::Backoff;

use super::{BytesBody, Connector};
use crate::Error;

pub type RequestJson<'a, Conn, T> = Request<'a, Conn, Json<T>>;
pub type RequestCollect<'a, Conn> = Request<'a, Conn, Collect>;

pub trait PollRequest {
    type Output;

    // Resets the request future, i.e killing an existing request if one is still pending,
    // then retrying the request.
    fn reset(self: Pin<&mut Self>);

    fn poll_request<Conn: Connector>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        parts: RequestParts<'_, Conn>,
    ) -> Poll<crate::Result<Self::Output>>;
}

pin_project_lite::pin_project! {
    pub struct Request<'a, Conn: Connector, Req: PollRequest = Base> {
        client: Cow<'a, HyperClient<Conn, BytesBody>>,
        request: http::Request<BytesBody>,
        #[pin]
        inner: Req,
    }
}

impl<Conn: Connector, Req: PollRequest + std::fmt::Debug> std::fmt::Debug
    for Request<'_, Conn, Req>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("client", &self.client.as_ref())
            .field("request", &self.request)
            .field("state", &self.inner)
            .finish()
    }
}

impl<Conn: Connector, Req: PollRequest> Request<'_, Conn, Req> {
    pub fn into_static(self) -> Request<'static, Conn, Req> {
        Request {
            client: Cow::Owned(self.client.into_owned()),
            request: self.request,
            inner: self.inner,
        }
    }

    pub fn reset(self: Pin<&mut Self>) {
        self.project().inner.reset()
    }
}

impl<'a, Conn: Connector> Request<'a, Conn, Base> {
    pub fn new(
        client: impl Into<Cow<'a, HyperClient<Conn, BytesBody>>>,
        request: http::Request<BytesBody>,
    ) -> Self {
        Self {
            client: client.into(),
            request,
            inner: Base::new(),
        }
    }

    pub fn collect(self) -> Request<'a, Conn, Collect> {
        Request {
            client: self.client,
            request: self.request,
            inner: Collect::Requesting {
                request: self.inner,
            },
        }
    }
}

impl<'a, Conn: Connector> Request<'a, Conn, Collect> {
    pub fn json<T>(self) -> Request<'a, Conn, Json<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        Request {
            client: self.client,
            request: self.request,
            inner: Json {
                fut: self.inner,
                _marker: PhantomData,
            },
        }
    }
}

impl<'a, Conn: Connector, Req: PollRequest> Future for Request<'a, Conn, Req> {
    type Output = Result<Req::Output, Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.inner.poll_request(
            cx,
            RequestParts {
                client: &**this.client,
                request: this.request,
            },
        )
    }
}

pub struct RequestParts<'a, Conn: Connector> {
    client: &'a HyperClient<Conn, BytesBody>,
    request: &'a mut http::Request<BytesBody>,
}

impl<Conn: Connector> RequestParts<'_, Conn> {
    fn reborrow(&mut self) -> RequestParts<'_, Conn> {
        RequestParts {
            client: self.client,
            request: self.request,
        }
    }

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
        tower::Service::poll_ready(&mut &*self.client, cx).map_err(Error::from)
    }

    fn call(&self) -> HyperResponseFuture {
        tower::Service::call(&mut &*self.client, self.request.clone())
    }
}

pin_project_lite::pin_project! {
    /// Top level type that makes a request, with retries and up to 1 redirect,
    /// and then tries to deserialize the response (from json) to [`T`].
    #[repr(transparent)]
    pub struct Json<T> {
        #[pin]
        fut: Collect,
        _marker: PhantomData<fn(T)>,
    }
}

impl<T> std::fmt::Debug for Json<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Json")
            .field("collect", &self.fut)
            .field("parse_as", &std::any::type_name::<T>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    /// Nearly top level type, that makes a request (with retries and up to 1 redirect),
    /// then collects the entire response body into a single, flat [`Bytes`].
    #[project = RequestCollectProjection]
    #[derive(Debug)]
    pub enum Collect {
        Requesting {
            #[pin]
            request: Base,
        },
        Collecting {
            parts: Option<(http::Uri, http::response::Parts)>,
            #[pin]
            collect: crate::util::CollectBody,
        }
    }
}

pin_project_lite::pin_project! {
    /// Makes a request with retries, and up to 1 redirect.
    #[project = BaseProjection]
    #[derive(Debug)]
    pub struct Base {
        has_redirected: bool,
        backoff: Option<Backoff>,
        // reusable sleep, that's created lazily when we backoff for
        // the first time.
        //
        // We do this instead of creating multiple sleeps for each backoff,
        // since if we need to backoff once, we'll likely need to do it again.
        // That, and wrapping in Pin<Box<>> lets us implement Unpin
        // unconditionally, which makes higher level futures easier and nicer
        // to implement.
        backoff_sleep: Option<Pin<Box<tokio::time::Sleep>>>,
        last_error: Option<Result<http::Response<Incoming>, Error>>,
        #[pin]
        state: RequestState,
    }
}

pin_project_lite::pin_project! {
    /// Internal type for the current state of the raw request, or backoff if
    /// we ran into an error.
    #[project = RequestStateProjection]
    #[derive(Debug)]
    enum RequestState {
        /// This variant isn't currently needed, since the hyper_util Client
        /// has no back-pressure mechanism (i.e <Client as Service>::poll_ready
        /// unconditionally returns Poll::Ready).
        ///
        /// If this ever changes, we'll run into errors, so assume we
        /// need to handle back-pressure.
        PollReady,
        Backoff,
        Requesting {
            #[pin]
            req: HyperResponseFuture,
        }
    }
}

impl BaseProjection<'_> {
    /// internal poll method that handles making requests and retrying on server errors
    /// (up to 5 times). Does __NOT__ handle making a redirect, that's handled in the main
    /// [`Request<'_, Conn> as Future>::poll`] call.
    fn poll_response<Conn: Connector>(
        &mut self,
        cx: &mut Context<'_>,
        parts: RequestParts<'_, Conn>,
    ) -> Poll<Result<http::Response<Incoming>, Error>> {
        use RequestStateProjection::{Backoff, PollReady, Requesting};

        loop {
            match self.state.as_mut().project() {
                Backoff => {
                    let sleep = self.backoff_sleep.as_mut().expect("invalid state");
                    std::task::ready!(sleep.as_mut().poll(cx));
                    self.state.set(RequestState::PollReady);
                }
                PollReady => {
                    std::task::ready!(parts.poll_ready(cx))?;
                    self.state
                        .set(RequestState::Requesting { req: parts.call() });
                }
                Requesting { req } => {
                    *self.last_error = Some(match std::task::ready!(req.poll(cx)) {
                        // Only retry on 5XX errors, since 4XX (client errors) are our fault,
                        // and 3XX (redirects) are handled by the caller.
                        Ok(resp) if resp.status().is_server_error() => Ok(resp),
                        Ok(resp) => return Poll::Ready(Ok(resp)),
                        Err(error) => Err(error.into()),
                    });

                    let backoff = self.backoff.get_or_insert_default();

                    let Some(backoff_once) = backoff.backoff_once() else {
                        return Poll::Ready(
                            self.last_error
                                .take()
                                .expect("we just set this")
                                .map_err(Error::from),
                        );
                    };

                    // do an initial poll with the sleep that gets returned,
                    // since it avoids one more loop (+ the associated state
                    // checking in 'get_backoff_sleep').
                    let sleep = backoff_once.insert_or_reset(self.backoff_sleep);
                    // we expect that a brand new or reset sleep would return
                    // pending, but just in case it does complete right away,
                    // treat it like we would in the Backoff branch
                    std::task::ready!(sleep.poll(cx));
                    self.state.set(RequestState::PollReady);
                }
            }
        }
    }
}

impl Base {
    pub(super) fn new() -> Self {
        Self {
            has_redirected: false,
            backoff: None,
            backoff_sleep: None,
            last_error: None,
            state: RequestState::PollReady,
        }
    }
}

impl PollRequest for Base {
    type Output = http::Response<Incoming>;

    #[inline]
    fn reset(mut self: Pin<&mut Self>) {
        self.set(Self::new());
    }

    #[inline]
    fn poll_request<Conn: Connector>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut parts: RequestParts<'_, Conn>,
    ) -> Poll<crate::Result<Self::Output>> {
        let mut this = self.project();

        loop {
            let response = std::task::ready!(this.poll_response(cx, parts.reborrow()))?;

            // handle up to 1 redirect, if we hit one
            if response.status().is_redirection() && !*this.has_redirected {
                let redirect_uri = get_redirect_uri(parts.request.uri(), response)?;

                tracing::debug!(message = "handling redirect...", ?redirect_uri, original_uri = ?parts.request.uri());

                *parts.request.uri_mut() = redirect_uri;

                // reset the inner future state
                this.state.set(RequestState::PollReady);
                *this.has_redirected = true;
                continue;
            }

            return Poll::Ready(Ok(response));
        }
    }
}

impl PollRequest for Collect {
    type Output = (http::response::Parts, Bytes);

    fn reset(mut self: Pin<&mut Self>) {
        self.set(Collect::Requesting {
            request: Base::new(),
        });
    }

    fn poll_request<Conn: Connector>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut parts: RequestParts<'_, Conn>,
    ) -> Poll<crate::Result<Self::Output>> {
        use RequestCollectProjection::{Collecting, Requesting};

        loop {
            match self.as_mut().project() {
                Requesting { mut request } => {
                    let response =
                        std::task::ready!(request.as_mut().poll_request(cx, parts.reborrow()))?;
                    let uri = parts.request.uri().clone();

                    let (parts, incoming) = response.into_parts();

                    self.as_mut().set(Self::Collecting {
                        parts: Some((uri, parts)),
                        collect: crate::util::collect_body(incoming),
                    });
                }
                Collecting { parts, collect } => {
                    let bytes = std::task::ready!(collect.poll(cx))?;
                    let (uri, parts) = parts.take().expect("polled after completion");

                    return Poll::Ready(if !parts.status.is_success() {
                        Err(Error::Response(Box::new(
                            crate::error::ResponseError::from_parts(uri, parts, bytes),
                        )))
                    } else {
                        Ok((parts, bytes))
                    });
                }
            }
        }
    }
}

impl<T> PollRequest for Json<T>
where
    T: serde::de::DeserializeOwned,
{
    type Output = (http::response::Parts, T);

    fn reset(self: Pin<&mut Self>) {
        self.project().fut.reset();
    }

    #[inline]
    fn poll_request<Conn: Connector>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        parts: RequestParts<'_, Conn>,
    ) -> Poll<crate::Result<Self::Output>> {
        let (parts, bytes) = std::task::ready!(self.project().fut.poll_request(cx, parts))?;
        let value: T = path_aware_serde::json::deserialize_slice(&bytes)?;
        Poll::Ready(Ok((parts, value)))
    }
}

fn get_redirect_uri(uri: &http::Uri, resp: http::Response<Incoming>) -> Result<http::Uri, Error> {
    debug_assert!(resp.status().is_redirection());

    let uri_header = match resp.headers().get(&http::header::LOCATION) {
        Some(header) => header,
        None => {
            return Err(Error::Response(Box::new(crate::ResponseError::from_parts(
                uri.clone(),
                resp.into_parts().0,
                Bytes::from_static(b"recieved redirect response with no 'location' header"),
            ))));
        }
    };

    match http::Uri::try_from(uri_header.as_bytes()) {
        Ok(uri) => Ok(uri),
        Err(error) => Err(Error::Response(Box::new(crate::ResponseError::from_parts(
            uri.clone(),
            resp.into_parts().0,
            Bytes::from(error.to_string()),
        )))),
    }
}
