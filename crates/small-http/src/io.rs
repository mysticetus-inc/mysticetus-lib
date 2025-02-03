use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, ToSocketAddrs};

pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct TokioIo {
        #[pin]
        stream: TcpStream,
    }
}

impl TokioIo {
    #[inline]
    pub const fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// convinience helper that calls [`TcpStream::connect`] under the hood
    #[inline]
    pub async fn tcp_connect(addr: impl ToSocketAddrs) -> io::Result<Self> {
        TcpStream::connect(addr).await.map(Self::new)
    }
}

impl hyper::rt::Read for TokioIo {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        // SAFETY: implementation copied from hyper-util, and given
        // that has millions of users it's probably implemented correctly
        let n = unsafe {
            let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
            match AsyncRead::poll_read(self.project().stream, cx, &mut tbuf) {
                Poll::Ready(Ok(())) => tbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(n);
        }

        Poll::Ready(Ok(()))
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

pub struct EmptyBody;

impl http_body::Body for EmptyBody {
    type Data = bytes::Bytes;
    type Error = std::convert::Infallible;

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(0)
    }

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TokioExec;

impl<Fut> hyper::rt::Executor<Fut> for TokioExec
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::spawn(fut);
    }
}
