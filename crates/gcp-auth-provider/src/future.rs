use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

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

impl<'a, G: Resolver> GetTokenFuture<'a, G> {
    pub(crate) fn new_http(client: &'a HttpClient<G>, request: http::Request<BytesBody>) -> Self {
        Self {
            inner: Inner::Http {
                req: client.request_json(request),
            },
        }
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
    pub(crate) fn new_gcloud(fut: crate::gcloud::GCloudFuture<Token>) -> Self {
        Self {
            inner: Inner::GCloud { fut },
        }
    }

    #[cfg(feature = "emulator")]
    pub(crate) const fn new_emulator() -> Self {
        Self {
            inner: Inner::Emulator,
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
                Inner::GCloud { fut } => Inner::GCloud { fut },
                #[cfg(feature = "emulator")]
                Inner::Emulator => Inner::Emulator,
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
            InnerProjection::GCloud { fut } => fut.poll(cx),
            #[cfg(feature = "emulator")]
            InnerProjection::Emulator => Poll::Ready(Ok(Token::EMULATOR_TOKEN)),
        }
    }
}

// the pin_project macro cant handle #[cfg(feature = "...")] on a specific variant,
// so we need to define a matrix for both relevant features.

// no gcloud or emulator
#[cfg(all(not(feature = "gcloud"), not(feature = "emulator")))]
pin_project_lite::pin_project! {
    #[project = InnerProjection]
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
    }
}

// only gcloud
#[cfg(all(feature = "gcloud", not(feature = "emulator")))]
pin_project_lite::pin_project! {
    #[project = InnerProjection]
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
        GCloud {
            #[pin]
            fut: crate::gcloud::GCloudFuture<Token>,
        }
    }
}

// only emulator
#[cfg(all(not(feature = "gcloud"), feature = "emulator"))]
pin_project_lite::pin_project! {
    #[project = InnerProjection]
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
        Emulator,
    }
}

// both gcloud and emulator
#[cfg(all(feature = "gcloud", feature = "emulator"))]
pin_project_lite::pin_project! {
    #[project = InnerProjection]
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
        GCloud {
            #[pin]
            fut: crate::gcloud::GCloudFuture<Token>,
        },
        Emulator,
    }
}
