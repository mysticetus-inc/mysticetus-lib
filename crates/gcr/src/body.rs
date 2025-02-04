use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::response::IntoResponse;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{Stream, StreamExt};
use http::{HeaderMap, StatusCode};
use http_body::Body;

const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum BodyError<E> {
    #[error(transparent)]
    Body(#[from] E),
    #[error("request payload exceeds 10MiB")]
    RequestTooLarge,
}

impl<E: fmt::Display> IntoResponse for BodyError<E> {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Body(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            Self::RequestTooLarge => {
                (StatusCode::BAD_REQUEST, "request payload exceeds 10MiB").into_response()
            }
        }
    }
}

pub trait CollectBodyExt<B: Body + Send + 'static>: Sized
where
    Bytes: From<B::Data>,
{
    fn collect_to_bytes(
        self,
    ) -> impl Future<Output = Result<Bytes, BodyError<B::Error>>> + Send + 'static;
}

impl<B: Body + Send + 'static> CollectBodyExt<B> for B
where
    Bytes: From<B::Data>,
{
    #[inline]
    fn collect_to_bytes(
        self,
    ) -> impl Future<Output = Result<Bytes, BodyError<<B as Body>::Error>>> + Send + 'static {
        collect_to_bytes(self, None)
    }
}

impl<B: Body + Send + 'static> CollectBodyExt<B> for http::Request<B>
where
    Bytes: From<B::Data>,
{
    #[inline]
    fn collect_to_bytes(
        self,
    ) -> impl Future<Output = Result<Bytes, BodyError<<B as Body>::Error>>> + Send + 'static {
        let content_len = try_parse_content_len(self.headers());
        collect_to_bytes(self.into_body(), content_len)
    }
}

impl<B: Body + Send + 'static> CollectBodyExt<B> for http::Response<B>
where
    Bytes: From<B::Data>,
{
    #[inline]
    fn collect_to_bytes(
        self,
    ) -> impl Future<Output = Result<Bytes, BodyError<<B as Body>::Error>>> + Send + 'static {
        let content_len = try_parse_content_len(self.headers());
        collect_to_bytes(self.into_body(), content_len)
    }
}

fn try_parse_content_len(headers: &HeaderMap) -> Option<usize> {
    headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|string| string.parse::<usize>().ok())
}

fn est_capacity(hint: http_body::SizeHint) -> usize {
    if let Some(exact) = hint.exact() {
        exact as usize
    } else if let Some(upper) = hint.upper() {
        upper as usize
    } else {
        match hint.lower() as usize {
            // if the body just returns the default 0 lower bound, "guess" 512 KiB
            0 => 512 * 1024,
            other => other,
        }
    }
}

pub async fn collect_to_bytes<B>(
    body: B,
    content_len: Option<usize>,
) -> Result<Bytes, BodyError<B::Error>>
where
    B: Body,
    Bytes: From<B::Data>,
{
    let hint = body.size_hint();

    let mut body_stream = std::pin::pin!(BodyStream::new(body));

    let Some(result) = body_stream.next().await else {
        return Ok(Bytes::new());
    };

    let mut first_chunk = Bytes::from(result?);

    // see if we can avoid copying bytes if the stream only has 1 body chunk
    let next_chunk = loop {
        match body_stream.next().await.transpose()? {
            // skip empty first chunks
            Some(chunk) if first_chunk.is_empty() => first_chunk = Bytes::from(chunk),
            Some(chunk) => break Bytes::from(chunk),
            None => return Ok(first_chunk),
        }
    };

    let chunks_len = first_chunk.len().saturating_add(next_chunk.len());

    if chunks_len > MAX_REQUEST_SIZE {
        return Err(BodyError::RequestTooLarge);
    }

    let capacity = content_len
        .unwrap_or_else(|| est_capacity(hint))
        .clamp(chunks_len, MAX_REQUEST_SIZE);

    // insert the first chunks into the new buffer
    let mut dst = BytesMut::with_capacity(capacity);
    dst.put(first_chunk);
    dst.put(next_chunk);

    while let Some(result) = body_stream.next().await {
        let chunk = Bytes::from(result?);

        // ignore empty chunks, and return an error if the new chunk would put
        // us over the 10MiB request size limit
        if chunk.is_empty() {
            continue;
        } else if dst.len().saturating_add(chunk.len()) > MAX_REQUEST_SIZE {
            return Err(BodyError::RequestTooLarge);
        }

        dst.put(chunk);
    }

    Ok(dst.freeze())
}

pin_project_lite::pin_project! {
    #[repr(transparent)]
    pub struct BodyStream<B> {
        #[pin]
        body: B,
    }
}

impl<B> BodyStream<B> {
    #[inline]
    pub fn new(body: B) -> Self {
        Self { body }
    }
}

impl<B> From<B> for BodyStream<B> {
    #[inline]
    fn from(value: B) -> Self {
        Self::new(value)
    }
}

impl<B> From<http::Request<B>> for BodyStream<B> {
    #[inline]
    fn from(value: http::Request<B>) -> Self {
        Self::new(value.into_body())
    }
}

impl<B> From<http::Response<B>> for BodyStream<B> {
    #[inline]
    fn from(value: http::Response<B>) -> Self {
        Self::new(value.into_body())
    }
}

impl<B: Body> Stream for BodyStream<B> {
    type Item = Result<B::Data, B::Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            let frame = match std::task::ready!(this.body.as_mut().poll_frame(cx)) {
                Some(Ok(frame)) => frame,
                Some(Err(error)) => return Poll::Ready(Some(Err(error))),
                None => return Poll::Ready(None),
            };

            if let Ok(data) = frame.into_data() {
                return Poll::Ready(Some(Ok(data)));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        #[inline(always)]
        const fn cast(int: u64) -> usize {
            int as usize
        }

        let hint = self.body.size_hint();

        (cast(hint.lower()), hint.upper().map(cast))
    }
}
