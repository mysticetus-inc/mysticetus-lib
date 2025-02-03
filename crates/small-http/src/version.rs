use std::future::Future;
use std::task::{Context, Poll};

#[cfg(feature = "http1")]
pub use http1::Http1;
#[cfg(feature = "http2")]
pub use http2::Http2;
use hyper::body::Incoming;

use crate::io::TokioIo;

#[cfg(feature = "http1")]
pub type DefaultHttpVersion = Http1;

#[cfg(all(feature = "http2", not(feature = "http1")))]
pub type DefaultHttpVersion = Http2;

#[cfg(all(not(feature = "http2"), not(feature = "http1")))]
compile_error!("one or both of the 'http1'/'http2' features must be enabled");

pub trait HttpVersion<B> {
    type SendRequest: SendRequest<B>;
    type Connection: Future<Output = hyper::Result<()>> + Send + 'static;
    type Builder: Builder<B, SendRequest = Self::SendRequest, Connection = Self::Connection>;

    #[inline]
    fn handshake(
        io: TokioIo,
    ) -> impl Future<Output = hyper::Result<(Self::SendRequest, Self::Connection)>> + Send + 'static
    {
        <Self::Builder as Builder<B>>::new().handshake(io)
    }
}

pub trait SendRequest<B>: Unpin + 'static {
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<hyper::Result<()>>;

    #[inline]
    fn ready(&mut self) -> impl Future<Output = hyper::Result<()>> + '_ {
        std::future::poll_fn(|cx| self.poll_ready(cx))
    }

    fn send_request(
        &mut self,
        req: http::Request<B>,
    ) -> impl Future<Output = hyper::Result<http::Response<Incoming>>>;
}

pub trait Builder<B>: Send {
    type SendRequest: SendRequest<B>;
    type Connection: Future<Output = hyper::Result<()>> + Send + 'static;

    fn new() -> Self;

    fn handshake(
        self,
        io: TokioIo,
    ) -> impl Future<Output = hyper::Result<(Self::SendRequest, Self::Connection)>> + Send + 'static;
}

#[cfg(feature = "http1")]
mod http1 {
    use std::future::Future;
    use std::task::{Context, Poll};

    use http_body::Body;
    use hyper::body::Incoming;
    use hyper::client::conn::http1;

    use super::{Builder, HttpVersion, SendRequest};
    use crate::io::TokioIo;

    pub enum Http1 {}

    impl<B> HttpVersion<B> for Http1
    where
        B: Body + Send + 'static,
        B::Data: Send,
        B::Error: std::error::Error + Send + Sync,
    {
        type Connection = http1::Connection<TokioIo, B>;
        type SendRequest = http1::SendRequest<B>;
        type Builder = http1::Builder;
    }

    impl<B> Builder<B> for http1::Builder
    where
        B: Body + Send + 'static,
        B::Data: Send,
        B::Error: std::error::Error + Send + Sync,
    {
        type SendRequest = http1::SendRequest<B>;
        type Connection = http1::Connection<TokioIo, B>;

        #[inline]
        fn new() -> Self {
            Self::new()
        }

        #[inline]
        fn handshake(
            self,
            io: TokioIo,
        ) -> impl Future<Output = hyper::Result<(Self::SendRequest, Self::Connection)>> + Send + 'static
        {
            Self::handshake(&self, io)
        }
    }

    impl<B: Body + 'static> SendRequest<B> for http1::SendRequest<B> {
        #[inline]
        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<hyper::Result<()>> {
            http1::SendRequest::poll_ready(self, cx)
        }

        #[inline]
        fn send_request(
            &mut self,
            req: http::Request<B>,
        ) -> impl Future<Output = hyper::Result<http::Response<Incoming>>> {
            http1::SendRequest::send_request(self, req)
        }
    }
}

#[cfg(feature = "http2")]
mod http2 {
    use hyper::body::Incoming;
    use hyper::client::conn::http2;

    use super::{Builder, ConnectionParts, HttpVersion, SendRequest};
    use crate::io::{TokioExec, TokioIo};

    pub enum Http2 {}

    impl<B> HttpVersion<B> for Http2
    where
        B: Body + Send + Unpin + 'static,
        B::Data: Send,
        B::Error: std::error::Error + Send + Sync,
    {
        type Connection = http2::Connection<TokioIo, B, TokioExec>;
        type SendRequest = http2::SendRequest<B>;
        type Builder = http2::Builder<TokioExec>;
    }

    impl<B> Builder<B> for http2::Builder<TokioExec>
    where
        B: Body + Send + Unpin + 'static,
        B::Data: Send,
        B::Error: std::error::Error + Send + Sync,
    {
        type Connection = http2::Connection<TokioIo, B, TokioExec>;
        type SendRequest = http2::SendRequest<B>;

        #[inline]
        fn new() -> Self {
            Self::new(TokioExec)
        }

        #[inline]
        fn handshake(
            self,
            io: TokioIo,
        ) -> impl Future<Output = hyper::Result<(Self::SendRequest, Self::Connection)>> + Send + 'static
        {
            Self::handshake(&self, io)
        }
    }

    impl<B: Body + 'static> SendRequest<B> for http2::SendRequest<B> {
        #[inline]
        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<hyper::Result<()>> {
            http2::SendRequest::poll_ready(self, cx)
        }

        #[inline]
        fn send_request(
            &mut self,
            req: http::Request<B>,
        ) -> impl Future<Output = hyper::Result<http::Response<Incoming>>> {
            http2::SendRequest::send_request(self, req)
        }
    }
}
