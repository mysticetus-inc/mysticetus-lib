use std::collections::VecDeque;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Buf;
use http::HeaderMap;
use http_body::Body;
use tokio::io::{AsyncBufRead, AsyncRead, ReadBuf};

pin_project_lite::pin_project! {
    #[project = BodyBufReaderProj]
    pub struct BodyBufReader<B: Body, Trailers: SendTrailers = IgnoreTrailers> {
        chunks: VecDeque<B::Data>,
        send_trailers: Trailers,
        #[pin]
        body: Option<B>,
    }
}

pub trait SendTrailers {
    fn send(&mut self, trailers: HeaderMap);
}

pub struct IgnoreTrailers;

impl SendTrailers for IgnoreTrailers {
    fn send(&mut self, _trailers: HeaderMap) {}
}

impl<B: Body, Trailers: SendTrailers> BodyBufReaderProj<'_, B, Trailers> {
    fn poll(&mut self, cx: &mut Context<'_>) -> Result<(), B::Error> {
        loop {
            let Some(body) = self.body.as_mut().as_pin_mut() else {
                break;
            };

            let frame = match body.poll_frame(cx) {
                Poll::Ready(Some(result)) => result?,
                Poll::Pending => break,
                Poll::Ready(None) => {
                    self.body.set(None);
                    break;
                }
            };

            match frame.into_data() {
                Ok(data) => self.chunks.push_back(data),
                Err(frame) => {
                    if let Ok(trailers) = frame.into_trailers() {
                        self.send_trailers.send(trailers);
                    }
                }
            }
        }

        Ok(())
    }
}

impl<B: Body, Trailers: SendTrailers> AsyncRead for BodyBufReader<B, Trailers> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut this = self.project();

        this.poll(cx)?;

        while let Some(front) = this.chunks.front_mut() {
            fill_read_buf(front, buf);

            if front.remaining() == 0 {
                this.chunks.pop_front();
            }

            if buf.remaining() == 0 {
                break;
            }
        }

        if this.body.is_some() || !this.chunks.is_empty() {
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

fn fill_read_buf<B: Buf + ?Sized>(src: &mut B, dst: &mut ReadBuf<'_>) {
    let mut remaining = dst.remaining();

    while 0 < remaining && 0 < src.remaining() {
        let chunk = src.chunk();
        let to_copy = chunk.len().min(remaining);
        dst.put_slice(&chunk[..to_copy]);
        remaining -= to_copy;
        src.advance(to_copy);
    }
}

impl<B: Body, Trailers: SendTrailers> AsyncBufRead for BodyBufReader<B, Trailers> {
    fn consume(self: Pin<&mut Self>, amt: usize) {
        if amt == 0 {
            return;
        }

        let this = self.project();

        let front = this
            .chunks
            .front_mut()
            .expect("consume called without having bytes available");

        front.advance(amt);

        if !front.has_remaining() {
            this.chunks.pop_front();
        }
    }

    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let mut this = self.project();
        this.poll(cx)?;

        match this.chunks.front() {
            Some(front) => Poll::Ready(Ok(front.chunk())),
            None => Poll::Ready(Ok(&[])),
        }
    }
}
