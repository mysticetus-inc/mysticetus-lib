use std::io;
use std::pin::Pin;
use std::sync::LazyLock;
use std::task::{Context, Poll};

use http::{HeaderName, HeaderValue, Uri};
use hyper::client::conn::http1::SendRequest;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task::JoinHandle;

use crate::error::Error;

static TOKEN_URI: LazyLock<Uri> = LazyLock::new(|| {
    Uri::from_static("/computeMetadata/v1/instance/service-accounts/default/token")
});

static PROJECT_ID_URI: LazyLock<Uri> =
    LazyLock::new(|| Uri::from_static("/computeMetadata/v1/project/project-id"));

const METADATA_FLAVOR_NAME: HeaderName = HeaderName::from_static("metadata-flavor");
const METADATA_FLAVOR_VALUE: HeaderValue = HeaderValue::from_static("google");

pub struct MetadataServer {
    driver: JoinHandle<hyper::Result<()>>,
    send_req: SendRequest<Empty>,
}

impl MetadataServer {
    pub async fn new() -> Result<Self, Error> {
        let host = tokio::net::lookup_host("metadata.google.internal:80")
            .await?
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "no hosts found for the internal metadata server",
                )
            })?;
        let stream = tokio::net::TcpStream::connect(host).await?;

        let (send_req, conn) = hyper::client::conn::http1::handshake(TokioIo { stream }).await?;

        Ok(Self {
            send_req,
            driver: tokio::spawn(conn),
        })
    }
}

pin_project_lite::pin_project! {
    struct TokioIo {
        #[pin]
        stream: tokio::net::TcpStream,
    }
}
impl hyper::rt::Read for TokioIo {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        unsafe {
            let mut tokio_buf = tokio::io::ReadBuf::uninit(buf.as_mut());
            std::task::ready!(self.project().stream.poll_read(cx, &mut tokio_buf))?;
            let written = tokio_buf.filled().len();
            buf.advance(written);
            Poll::Ready(Ok(()))
        }
    }
}

impl hyper::rt::Write for TokioIo {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        AsyncWrite::poll_write(self.project().stream, cx, buf)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        AsyncWrite::poll_flush(self.project().stream, cx)
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        AsyncWrite::poll_shutdown(self.project().stream, cx)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        AsyncWrite::is_write_vectored(&self.stream)
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        AsyncWrite::poll_write_vectored(self.project().stream, cx, bufs)
    }
}

pub struct Empty;

impl http_body::Body for Empty {
    type Data = bytes::Bytes;
    type Error = std::convert::Infallible;

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(0)
    }

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        std::task::Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn test_metadata() -> crate::Result<()> {
    let server = MetadataServer::new().await?;

    Ok(())
}
