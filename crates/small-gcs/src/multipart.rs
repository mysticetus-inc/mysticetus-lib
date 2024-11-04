#![allow(dead_code)] // in dev

use std::borrow::Cow;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{BufMut, Bytes, BytesMut};
use futures::Stream;
use reqwest::Body;

use crate::{Error, NewObject};

const DEFAULT_BOUNDARY: &str = "--small-gcs-bound";

const JSON_TYPE: &str = "Content-Type: application/json; charset=UTF-8\r\n";

const HTTP_NEWLINE: &[u8] = b"\r\n";

const MIN_SIZE: usize = 3 * DEFAULT_BOUNDARY.len() + JSON_TYPE.len() + 10 + 2;

pub struct MultipartBuilder<C> {
    buf: BytesMut,
    content: C,
}

impl<C> MultipartBuilder<C> {
    pub fn new<S, M>(metadata: &NewObject<S, M>) -> Result<Self, Error>
    where
        S: AsRef<str>,
        NewObject<S, M>: serde::Serialize,
    {
    }
}

pub trait IntoBody {
    fn into_body(self) -> Body;
}

/*


pin_project_lite::pin_project! {
    pub struct Multipart<M, S> {
        boundary: Cow<'static, str>,
        metadata: M,
        #[pin]
        content_stream: S,
        content_len: u64,
        content_mime: mime_guess::Mime,
        bytes_uploaded: u64,
        state: State,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    Init,
    Content,
    Done,
}

impl<M, S> Multipart<M, S> {
    pub fn new(
        metadata: M,
        content_stream: S,
        content_len: u64,
        content_mime: mime_guess::Mime,
    ) -> Self {
        Self {
            boundary: Cow::Borrowed(DEFAULT_BOUNDARY),
            metadata,
            content_len,
            bytes_uploaded: 0,
            content_mime,
            content_stream,
            state: State::Init,
        }
    }
}

impl<M, S, O, E> Stream for Multipart<M, S>
where
    M: serde::Serialize,
    S: Stream<Item = Result<O, E>>,
    bytes::Bytes: From<O>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match *this.state {
            State::Init => {
                let mut leading = BytesMut::with_capacity(1024);

                leading.put_slice(b"--");
                leading.put_slice(this.boundary.as_bytes());
                leading.put_slice(JSON_TYPE.as_bytes());

                if let Err(error) = serde_json::to_writer((&mut leading).writer(), this.metadata) {
                    return Poll::Ready(Some(Err(error.into())));
                }
                leading.put_slice(HTTP_NEWLINE);

                leading.put_slice(b"--");
                leading.put_slice(this.boundary.as_bytes());
                leading.put_slice(HTTP_NEWLINE);

                leading.put_slice(b"Content-Type: ");
                leading.put_slice(this.content_mime.essence_str().as_bytes());
                leading.put_slice(HTTP_NEWLINE);

                *this.state = State::Content;

                Poll::Ready(Some(Ok(leading.freeze())))
            }
            State::Content => {
                match this.content_stream.poll_next(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Some(Ok(bytes))) => {
                        let b = Bytes::from(bytes);
                        println!("{} of {} bytes", this.bytes_uploaded, this.content_len);
                        *this.bytes_uploaded += b.len() as u64;
                        Poll::Ready(Some(Ok(b)))
                    }
                    Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
                    Poll::Ready(None) => {
                        // set to done, since this last chunk is the last chunk we're sending
                        *this.state = State::Done;

                        let mut last_line =
                            BytesMut::with_capacity(HTTP_NEWLINE.len() + 2 + this.boundary.len());
                        last_line.extend_from_slice(HTTP_NEWLINE);
                        last_line.extend_from_slice(b"--");
                        last_line.extend_from_slice(this.boundary.as_bytes());

                        Poll::Ready(Some(Ok(last_line.freeze())))
                    }
                }
            }
            State::Done => return Poll::Ready(None),
        }
    }
}
*/
