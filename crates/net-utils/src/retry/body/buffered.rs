use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes};
use http_body::{Body, Frame};

use crate::retry::body::buf_list::BufList;

pub struct BufferedBody {
    chunks: BufList,
    trailers: Option<http::HeaderMap>,
}

impl BufferedBody {
    pub fn new(chunks: BufList, trailers: Option<http::HeaderMap>) -> Self {
        Self { chunks, trailers }
    }

    pub fn pop_frame(&mut self) -> Option<Frame<Bytes>> {
        if let Some(data) = self.chunks.pop() {
            return Some(Frame::data(data));
        }

        if let Some(trailers) = self.trailers.take() {
            return Some(Frame::trailers(trailers));
        }

        None
    }
}

impl Body for BufferedBody {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();

        let Some(frame) = this.pop_frame() else {
            return Poll::Ready(None);
        };

        if frame.is_data() && !this.is_end_stream() {
            cx.waker().wake_by_ref();
        }

        Poll::Ready(Some(Ok(frame)))
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        !self.chunks.has_remaining() && self.trailers.is_none()
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.chunks.remaining() as u64)
    }
}
