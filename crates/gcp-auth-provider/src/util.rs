use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{BufMut, Bytes, BytesMut};
use http_body::Body;
use hyper::body::Incoming;
use tokio::task::JoinHandle;

pin_project_lite::pin_project! {
    #[project = CollectBodyProjection]
    #[derive(Debug)]
    pub(crate) struct CollectBody {
        #[pin]
        body: Incoming,
        state: State,
    }
}

pub(crate) fn collect_body(body: Incoming) -> CollectBody {
    CollectBody {
        body,
        state: State::Pending,
    }
}

#[derive(Debug)]
enum State {
    Pending,
    FirstChunk { buf: Bytes },
    CopyingRemaining { dst: BytesMut },
}

impl Future for CollectBody {
    type Output = Result<Bytes, hyper::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let maybe_chunk = std::task::ready!(this.poll_frame(cx))?;

            *this.state = match (&mut *this.state, maybe_chunk) {
                (State::Pending, None) => return Poll::Ready(Ok(Bytes::new())),
                (State::Pending, Some(buf)) => State::FirstChunk { buf },
                (State::FirstChunk { buf }, None) => return Poll::Ready(Ok(std::mem::take(buf))),
                (State::FirstChunk { buf }, Some(second_chunk)) => {
                    let first_chunk = std::mem::take(buf);
                    let mut dst = match first_chunk.try_into_mut() {
                        Ok(dst) => dst,
                        Err(bytes) => {
                            let mut dst = BytesMut::with_capacity(bytes.len() + second_chunk.len());
                            dst.put(bytes);
                            dst
                        }
                    };

                    dst.put(second_chunk);

                    State::CopyingRemaining { dst }
                }
                (State::CopyingRemaining { dst }, Some(chunk)) => {
                    dst.put(chunk);
                    continue;
                }
                (State::CopyingRemaining { dst }, None) => {
                    let buf = std::mem::take(dst).freeze();
                    return Poll::Ready(Ok(buf));
                }
            }
        }
    }
}

impl CollectBodyProjection<'_> {
    fn poll_frame(&mut self, cx: &mut Context<'_>) -> Poll<Result<Option<Bytes>, hyper::Error>> {
        loop {
            let frame = match std::task::ready!(self.body.as_mut().poll_frame(cx)) {
                Some(frame) => frame?,
                None => return Poll::Ready(Ok(None)),
            };

            if let Ok(data) = frame.into_data() {
                return Poll::Ready(Ok(Some(data)));
            }
        }
    }
}

pub struct ReadFuture {
    handle: JoinHandle<std::io::Result<Vec<u8>>>,
}

impl ReadFuture {
    pub fn read(path: impl Into<PathBuf>) -> Self {
        fn spawn(path: PathBuf) -> JoinHandle<std::io::Result<Vec<u8>>> {
            tokio::task::spawn_blocking(move || std::fs::read(path))
        }

        Self {
            handle: spawn(path.into()),
        }
    }
}

impl Future for ReadFuture {
    type Output = std::io::Result<Vec<u8>>;
    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io;

        match std::task::ready!(Pin::new(&mut self.get_mut().handle).poll(cx)) {
            Ok(result) => Poll::Ready(result),
            Err(err) => Poll::Ready(Err(io::Error::other(err))),
        }
    }
}

pub(crate) enum CowMut<'a, T> {
    RefMut(&'a mut T),
    Owned(T),
}

impl<T> CowMut<'_, Option<T>> {
    pub fn take_into_static(self) -> CowMut<'static, Option<T>> {
        match self {
            Self::Owned(owned) => CowMut::Owned(owned),
            Self::RefMut(refer) => CowMut::Owned(refer.take()),
        }
    }
}

impl<T> std::ops::Deref for CowMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(o) => o,
            Self::RefMut(r) => r,
        }
    }
}

impl<T> std::ops::DerefMut for CowMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(o) => o,
            Self::RefMut(r) => r,
        }
    }
}




