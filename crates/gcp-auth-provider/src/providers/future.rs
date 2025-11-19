use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::TryMaybeDone;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::connect::dns::{GaiResolver, Name};
use tower::Service;

use crate::Error;
use crate::client::future::RequestJson;
use crate::client::{BytesBody, HttpClient, HttpsClient};
use crate::token::{Bearer, Token};

pin_project_lite::pin_project! {
    pub struct GetTokenFuture<'a, G: Resolver = GaiResolver> {
        #[pin]
        inner: Inner<'a, G>,
    }
}

// the pin_project macro cant handle #[cfg(feature = "...")],
// so we need to use the full proc-macro based impl.
#[pin_project::pin_project(project = InnerProjection)]
enum Inner<'a, G: Resolver = GaiResolver> {
    Error {
        error: Option<crate::Error>,
    },
    Http {
        #[pin]
        req: RequestJson<'a, HttpConnector<G>, Token<Bearer>>,
    },
    Https {
        #[pin]
        req: RequestJson<'a, HttpsConnector<HttpConnector<G>>, Token<Bearer>>,
    },
    #[cfg(feature = "gcloud")]
    GCloud {
        gcloud: std::borrow::Cow<'a, super::gcloud::GCloudProvider>,
        #[pin]
        future: super::gcloud::GCloudFuture<Token>,
    },
    #[cfg(feature = "emulator")]
    Emulator,
    #[cfg(feature = "pinned-token-future")]
    Pinned {
        pinned: Pin<Box<dyn Future<Output = Result<Token, Error>> + Send + 'static>>,
    },
}

impl<'a, G: Resolver> GetTokenFuture<'a, G> {
    pub(crate) fn new_http(client: &'a HttpClient<G>, request: http::Request<BytesBody>) -> Self {
        Self {
            inner: Inner::Http {
                req: client.request_json(request),
            },
        }
    }

    pub(crate) fn new_http_request(req: RequestJson<'a, HttpConnector<G>, Token<Bearer>>) -> Self {
        Self {
            inner: Inner::Http { req },
        }
    }

    #[cfg(feature = "pinned-token-future")]
    pub fn pin(future: impl Future<Output = Result<Token, Error>> + Send + 'static) -> Self {
        Self::from_pinned(Box::pin(future))
    }

    #[cfg(feature = "pinned-token-future")]
    pub fn from_pinned(
        pinned: Pin<Box<dyn Future<Output = Result<Token, Error>> + Send + 'static>>,
    ) -> Self {
        Self {
            inner: Inner::Pinned { pinned },
        }
    }

    pub(crate) fn try_maybe_http(
        req: TryMaybeDone<RequestJson<'a, HttpConnector<G>, Token<Bearer>>>,
    ) -> TryMaybeDone<Self> {
        super::map_token_future(req, Self::new_http_request, |(_, token)| {
            token.into_unit_token_type()
        })
    }

    pub(crate) fn new_https(client: &'a HttpsClient<G>, request: http::Request<BytesBody>) -> Self {
        Self {
            inner: Inner::Https {
                req: client.request_json(request),
            },
        }
    }

    pub(crate) fn new_error(error: Error) -> Self {
        Self {
            inner: Inner::Error { error: Some(error) },
        }
    }

    #[cfg(feature = "gcloud")]
    pub(crate) fn new_gcloud(gcloud: &'a super::gcloud::GCloudProvider) -> Self {
        Self {
            inner: Inner::GCloud {
                future: gcloud.get_token_inner(),
                gcloud: std::borrow::Cow::Borrowed(gcloud),
            },
        }
    }

    #[cfg(feature = "emulator")]
    pub(crate) const fn new_emulator() -> Self {
        Self {
            inner: Inner::Emulator,
        }
    }

    /// Resets the inner future in an attmept to retry. Should not be attmepted
    /// if the error in a previous attempt returns true for [`Error::is_fatal`].
    pub fn reset(mut self: Pin<&mut Self>) {
        let this = self.as_mut().project();

        use InnerProjection::*;

        match this.inner.project() {
            Http { req } => req.reset(),
            Https { req } => req.reset(),
            #[cfg(feature = "gcloud")]
            GCloud { gcloud, mut future } => {
                future.set(gcloud.get_token_inner());
            }
            _ => (),
        }
    }

    pub fn into_static(self) -> GetTokenFuture<'static, G> {
        GetTokenFuture {
            inner: match self.inner {
                Inner::Error { error } => Inner::Error { error },
                Inner::Http { req } => Inner::Http {
                    req: req.into_static(),
                },
                Inner::Https { req } => Inner::Https {
                    req: req.into_static(),
                },
                #[cfg(feature = "gcloud")]
                Inner::GCloud { gcloud, future } => Inner::GCloud {
                    gcloud: std::borrow::Cow::Owned(gcloud.into_owned()),
                    future,
                },
                #[cfg(feature = "emulator")]
                Inner::Emulator => Inner::Emulator,
                #[cfg(feature = "pinned-token-future")]
                Inner::Pinned { pinned } => Inner::Pinned { pinned },
            },
        }
    }

    pub fn block_on_current(self) -> <Self as Future>::Output {
        tokio::runtime::Handle::current().block_on(std::pin::pin!(self))
    }

    pub fn block_on(self, runtime: &tokio::runtime::Runtime) -> <Self as Future>::Output {
        runtime.block_on(std::pin::pin!(self))
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<<Self as Future>::Output> {
        tokio::spawn(self.into_static())
    }
}

type BoxErr = Box<dyn std::error::Error + Send + Sync>;

pub trait Resolver:
    Service<Name, Future: Send, Response: Iterator<Item = SocketAddr>, Error: Into<BoxErr>>
    + Clone
    + Send
    + Sync
    + 'static
{
}

impl<T> Resolver for T
where
    T: Service<Name>,
    T::Future: Send,
    T::Response: Iterator<Item = SocketAddr>,
    T::Error: Into<BoxErr>,
    T: Clone + Send + Sync + 'static,
{
}

impl<G> Future for GetTokenFuture<'_, G>
where
    G: Resolver,
{
    type Output = Result<Token, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            InnerProjection::Error { error } => {
                let error = error.take().expect("polled after completion");
                Poll::Ready(Err(error))
            }
            InnerProjection::Http { req } => {
                let (_, token) = std::task::ready!(req.poll(cx))?;
                Poll::Ready(Ok(token.into_unit_token_type()))
            }
            InnerProjection::Https { req } => {
                let (_, token) = std::task::ready!(req.poll(cx))?;
                Poll::Ready(Ok(token.into_unit_token_type()))
            }
            #[cfg(feature = "gcloud")]
            InnerProjection::GCloud { future, .. } => future.poll(cx),
            #[cfg(feature = "emulator")]
            InnerProjection::Emulator => Poll::Ready(Ok(Token::EMULATOR_TOKEN)),
            #[cfg(feature = "pinned-token-future")]
            InnerProjection::Pinned { pinned } => pinned.as_mut().poll(cx),
        }
    }
}

impl std::fmt::Debug for GetTokenFuture<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Inner::Error { error: Some(error) } => f
                .debug_tuple("GetTokenFuture::Error")
                .field(&error)
                .finish(),
            Inner::Error { error: None } => f
                .debug_tuple("GetTokenFuture::Error")
                .finish_non_exhaustive(),
            Inner::Http { req } => f.debug_tuple("GetTokenFuture::Http").field(&req).finish(),
            Inner::Https { req } => f.debug_tuple("GetTokenFuture::Https").field(&req).finish(),
            #[cfg(feature = "gcloud")]
            Inner::GCloud { future, gcloud } => f
                .debug_struct("GetTokenFuture::GCloud")
                .field("future", &future)
                .field("gcloud", &gcloud)
                .finish(),
            #[cfg(feature = "emulator")]
            Inner::Emulator => f.pad("GetTokenFuture::Emulator"),
            #[cfg(feature = "pinned-token-future")]
            Inner::Pinned { .. } => f
                .debug_struct("GetTokenFuture::Pinned")
                .finish_non_exhaustive(),
        }
    }
}
