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

pin_project_lite::pin_project! {
    /// Top level type that makes a request, with retries and up to 1 redirect,
    /// and then tries to deserialize the response (from json) to [`T`].
    pub struct RequestJson<'a, Conn: Connector, T> {
        #[pin]
        fut: RequestCollect<'a, Conn>,
        _marker: PhantomData<fn(T)>,
    }
}

pin_project_lite::pin_project! {
    /// Nearly top level type, that makes a request (with retries and up to 1 redirect),
    /// then collects the entire response body into a single, flat [`Bytes`].
    #[project = RequestCollectProjection]
    pub enum RequestCollect<'a, Conn: Connector> {
        Requesting {
            #[pin]
            request: Request<'a, Conn>,
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
    #[project = RequestProjection]
    pub struct Request<'a, Conn: Connector> {
        client: Cow<'a, HyperClient<Conn, BytesBody>>,
        request: http::Request<BytesBody>,
        has_redirected: bool,
        backoff: Option<Backoff>,
        last_error: Option<Result<http::Response<Incoming>, Error>>,
        #[pin]
        state: RequestState,
    }
}

pin_project_lite::pin_project! {
    /// Internal type for the current state of the raw request, or backoff if
    /// we ran into an error.
    #[project =  RequestStateProjection]
    enum RequestState {
        /// This variant isn't currently needed, since the hyper_util Client
        /// has no backpressure mechanism (i.e <Client as Service>::poll_ready
        /// unconditionally returns Poll::Ready).
        ///
        /// If this ever changes, we'll run into errors, so assume we
        /// need to handle backpressure.
        PollReady,
        Backoff {
            #[pin]
            sleep: tokio::time::Sleep,
        },
        Requesting {
            #[pin]
            req: HyperResponseFuture,
        }
    }
}

impl<Conn: Connector> RequestProjection<'_, '_, Conn> {
    /// internal poll method that handles making requests and retrying on server errors
    /// (up to 5 times). Does __NOT__ handle making a redirect, that's handled in the main
    /// [`Request<'_, Conn> as Future>::poll`] call.
    fn poll_response(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<http::Response<Incoming>, Error>> {
        use RequestStateProjection::{Backoff, PollReady, Requesting};

        loop {
            match self.state.as_mut().project() {
                Backoff { sleep } => {
                    std::task::ready!(sleep.poll(cx));
                    self.state.set(RequestState::PollReady);
                }
                PollReady => {
                    std::task::ready!(poll_ready(&**self.client, cx))?;
                    self.state.set(RequestState::Requesting {
                        req: call(&**self.client, self.request),
                    });
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

                    let Some(sleep) = backoff.backoff_once() else {
                        return Poll::Ready(
                            self.last_error
                                .take()
                                .expect("we just set this")
                                .map_err(Error::from),
                        );
                    };

                    self.state.set(RequestState::Backoff {
                        sleep: sleep.into_future(),
                    });
                }
            }
        }
    }
}

impl<'a, Conn: Connector> Request<'a, Conn> {
    pub(super) fn new(
        client: &'a HyperClient<Conn, BytesBody>,
        request: http::Request<BytesBody>,
    ) -> Self {
        Self {
            request,
            has_redirected: false,
            client: Cow::Borrowed(client),
            backoff: None,
            last_error: None,
            state: RequestState::PollReady,
        }
    }
}
impl<'a, Conn: Connector> Request<'a, Conn> {
    pub(crate) fn into_static(self) -> Request<'static, Conn> {
        Request {
            request: self.request,
            has_redirected: self.has_redirected,
            client: Cow::Owned(self.client.into_owned()),
            backoff: self.backoff,
            last_error: self.last_error,
            state: self.state,
        }
    }
}

impl<'a, Conn: Connector> Future for Request<'a, Conn> {
    type Output = Result<http::Response<Incoming>, Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let response = std::task::ready!(this.poll_response(cx))?;

            // handle up to 1 redirect, if we hit one
            if response.status().is_redirection() && !*this.has_redirected {
                let redirect_uri = get_redirect_uri(this.request.uri(), response)?;

                tracing::debug!(message = "handling redirect...", ?redirect_uri, original_uri = ?this.request.uri());

                *this.request.uri_mut() = redirect_uri;

                // reset the inner future state
                this.state.set(RequestState::PollReady);
                *this.has_redirected = true;
                continue;
            }

            return Poll::Ready(Ok(response));
        }
    }
}

impl<'a, Conn: Connector> RequestCollect<'a, Conn> {
    pub(crate) fn json<T>(self) -> RequestJson<'a, Conn, T> {
        RequestJson {
            fut: self,
            _marker: PhantomData,
        }
    }
}

impl<'a, Conn: Connector> RequestCollect<'a, Conn> {
    pub(crate) fn into_static(self) -> RequestCollect<'static, Conn> {
        match self {
            Self::Collecting { parts, collect } => RequestCollect::Collecting { parts, collect },
            Self::Requesting { request } => RequestCollect::Requesting {
                request: request.into_static(),
            },
        }
    }
}

impl<'a, Conn: Connector> Future for RequestCollect<'a, Conn> {
    type Output = Result<(http::response::Parts, Bytes), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use RequestCollectProjection::{Collecting, Requesting};

        loop {
            match self.as_mut().project() {
                Requesting { mut request } => {
                    let response = std::task::ready!(request.as_mut().poll(cx))?;
                    let uri = request.project().request.uri().clone();

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
                        Err(Error::Response(crate::error::ResponseError::from_parts(
                            uri, parts, bytes,
                        )))
                    } else {
                        Ok((parts, bytes))
                    });
                }
            }
        }
    }
}

impl<'a, Conn: Connector, T> RequestJson<'a, Conn, T> {
    pub(crate) fn into_static(self) -> RequestJson<'static, Conn, T> {
        RequestJson {
            fut: self.fut.into_static(),
            _marker: PhantomData,
        }
    }
}

impl<'a, Conn: Connector, T> Future for RequestJson<'a, Conn, T>
where
    T: serde::de::DeserializeOwned,
{
    type Output = Result<(http::response::Parts, T), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (parts, bytes) = std::task::ready!(self.project().fut.poll(cx))?;
        let value: T = path_aware_serde::json::deserialize_slice(&bytes)?;
        Poll::Ready(Ok((parts, value)))
    }
}

// helper methods to call tower::Service methods on shared refernces

#[inline]
fn poll_ready<'a, Client>(mut client: &'a Client, cx: &mut Context<'_>) -> Poll<Result<(), Error>>
where
    &'a Client: tower::Service<http::Request<super::BytesBody>, Error: Into<Error>>,
{
    <&'a Client as tower::Service<http::Request<super::BytesBody>>>::poll_ready(&mut client, cx)
        .map_err(Into::into)
}

#[inline]
fn call<'a, Client>(
    mut client: &'a Client,
    request: &http::Request<BytesBody>,
) -> <&'a Client as tower::Service<http::Request<BytesBody>>>::Future
where
    &'a Client: tower::Service<http::Request<BytesBody>>,
{
    <&'a Client as tower::Service<http::Request<BytesBody>>>::call(&mut client, request.clone())
}

fn get_redirect_uri(uri: &http::Uri, resp: http::Response<Incoming>) -> Result<http::Uri, Error> {
    debug_assert!(resp.status().is_redirection());

    let uri_header = match resp.headers().get(&http::header::LOCATION) {
        Some(header) => header,
        None => {
            return Err(Error::Response(crate::ResponseError::from_parts(
                uri.clone(),
                resp.into_parts().0,
                Bytes::from_static(b"recieved redirect response with no 'location' header"),
            )));
        }
    };

    match http::Uri::try_from(uri_header.as_bytes()) {
        Ok(uri) => Ok(uri),
        Err(error) => Err(Error::Response(crate::ResponseError::from_parts(
            uri.clone(),
            resp.into_parts().0,
            Bytes::from(error.to_string()),
        ))),
    }
}
