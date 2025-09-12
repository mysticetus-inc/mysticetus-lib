use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{Stream, StreamExt};
use itoa::Integer;
use net_utils::backoff::Backoff;
use num_traits::PrimInt;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use reqwest::{RequestBuilder, header};

use crate::client::Client;
use crate::params::Alt;
use crate::{Error, Object};

pub struct ReadBuilder<'a, R = std::ops::RangeFull, I = u64> {
    builder: RequestBuilder,
    range: R,
    _range_int_type: PhantomData<fn(I)>,
    shared: &'a Client,
}

impl<'a> ReadBuilder<'a> {
    #[inline]
    pub(super) fn new(shared: &'a Client, bucket: &str, path: &str) -> Self {
        let url = crate::url::UrlBuilder::new(bucket).name(path).format();

        ReadBuilder {
            builder: shared.client.get(url),
            shared,
            _range_int_type: PhantomData,
            range: ..,
        }
    }

    #[inline]
    pub(super) fn new_buf(
        shared: &'a Client,
        url_buf: &mut String,
        bucket: &str,
        path: &str,
    ) -> Self {
        crate::url::UrlBuilder::new(bucket)
            .name(path)
            .format_into(url_buf);

        ReadBuilder {
            builder: shared.client.get(url_buf.as_str()),
            _range_int_type: PhantomData,
            shared,
            range: ..,
        }
    }
}

impl<'a, R, I> ReadBuilder<'a, R, I>
where
    R: RangeBounds<I>,
    I: Integer + PrimInt,
{
    #[inline]
    fn with_query_param<S>(self, field: &str, value: S) -> Self
    where
        S: serde::Serialize,
    {
        Self {
            builder: self.builder.query(&[(field, value)]),
            shared: self.shared,
            range: self.range,
            _range_int_type: PhantomData,
        }
    }

    #[inline]
    pub fn generation(self, generation: i64) -> Self {
        self.with_query_param("generation", generation)
    }

    #[inline]
    pub fn if_generation_match(self, generation: i64) -> Self {
        self.with_query_param("ifGenerationMatch", generation)
    }

    #[inline]
    pub fn if_generation_not_match(self, generation: i64) -> Self {
        self.with_query_param("ifGenerationNotMatch", generation)
    }

    pub fn range<Range, Int>(self, range: Range) -> ReadBuilder<'a, Range, Int>
    where
        Range: RangeBounds<Int>,
        Int: Integer + PrimInt,
    {
        ReadBuilder {
            builder: self.builder,
            shared: self.shared,
            _range_int_type: PhantomData,
            range,
        }
    }

    async fn send(self, alt: Alt) -> Result<(reqwest::Response, Option<u64>), Error> {
        let auth_header = self.shared.auth.get_header().await?;

        let mut builder = self
            .builder
            .header(header::AUTHORIZATION, auth_header)
            .query(&[alt]);

        let size_hint = match make_range_header(self.range)? {
            Some((header, size_hint)) => {
                builder = builder.header(header::RANGE, header);
                size_hint
            }
            None => None,
        };

        let request = builder.build()?;

        let resp = crate::try_execute_with_backoff(&self.shared, request, Backoff::default).await?;

        let resp = crate::validate_response(resp).await?;

        Ok((resp, size_hint))
    }

    pub async fn metadata(self) -> Result<Object, Error> {
        let (resp, _size_hint) = self
            .with_query_param("fields", Object::FIELDS)
            .send(Alt::Json)
            .await?;

        resp.json().await.map_err(Error::Reqwest)
    }

    pub async fn metadata_opt(self) -> Result<Option<Object>, Error> {
        match self.metadata().await {
            Ok(obj) => Ok(Some(obj)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    pub async fn content(
        self,
    ) -> Result<ReadStream<impl Stream<Item = Result<Bytes, Error>>>, Error> {
        use futures::TryStreamExt;

        let (mut resp, size_hint) = self.send(Alt::Media).await?;

        let headers = std::mem::take(resp.headers_mut());

        Ok(ReadStream {
            stream: resp.bytes_stream().map_err(Error::Reqwest),
            headers,
            size_hint,
        })
    }

    pub async fn content_opt(
        self,
    ) -> Result<Option<ReadStream<impl Stream<Item = Result<Bytes, Error>>>>, Error> {
        match self.content().await {
            Ok(stream) => Ok(Some(stream)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    async fn with_chunks<F, E>(self, mut f: F) -> Result<usize, Error>
    where
        F: FnMut(Bytes) -> Result<(), E>,
        Error: From<E>,
    {
        let stream = self.content().await?;
        futures::pin_mut!(stream);
        let mut total = 0;

        while let Some(result) = stream.next().await {
            let chunk = result?;
            total += chunk.len();
            f(chunk)?;
        }

        Ok(total)
    }

    pub async fn async_write_content<W>(self, writer: W) -> Result<usize, Error>
    where
        W: tokio::io::AsyncWrite,
    {
        let stream = self.content().await?;
        futures::pin_mut!(stream);
        futures::pin_mut!(writer);

        let mut chunks = std::collections::VecDeque::with_capacity(16);
        let mut total = 0;

        let mut is_done = false;

        futures::future::poll_fn(move |cx| {
            // if the stream isn't completed, poll to get another chunk, and ignore if it's pending.
            if !is_done {
                if let Poll::Ready(item) = stream.as_mut().poll_next(cx) {
                    match item {
                        Some(Ok(chunk)) => chunks.push_back(chunk),
                        Some(Err(error)) => return Poll::Ready(Err(error)),
                        None => is_done = true,
                    }
                }
            }

            // do this in a loop so we can write multiple chunks if the writer can handle it.
            loop {
                // get the current front chunk, if there isnt one, return if the stream is finished,
                // since that means we're done. otherwise, return pending since we're waiting
                // on the stream.
                let chunk = match chunks.front_mut() {
                    Some(chunk) => chunk,
                    None if is_done => return Poll::Ready(Ok(total)),
                    None => return Poll::Pending,
                };

                // inner loop to (try) and drain the current chunk.
                loop {
                    // if the writer isnt ready, we need to bail as pending
                    match writer.as_mut().poll_write(cx, chunk.chunk()) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Err(err)) => return Poll::Ready(Err(err.into())),
                        Poll::Ready(Ok(bytes_written)) => {
                            total += bytes_written;
                            chunk.advance(bytes_written);
                            // if the chunk has been drained, we can remove it and start again at
                            // the top of the outer loop with the next chunk (if there is one),
                            // otherwise keep trying to write this chunk.
                            if !chunk.has_remaining() {
                                chunks.pop_front();
                                break;
                            }
                        }
                    }
                }
            }
        })
        .await
    }

    pub async fn write_content<W>(self, writer: &mut W) -> Result<usize, Error>
    where
        W: std::io::Write,
    {
        self.with_chunks(|chunk| writer.write_all(&chunk)).await
    }

    pub async fn fill_buf_with_content<B>(self, buf: &mut B) -> Result<usize, Error>
    where
        B: BufMut,
    {
        let f = |chunk| {
            buf.put(chunk);
            Ok(()) as Result<(), Error>
        };

        self.with_chunks(f).await
    }

    pub async fn content_to_bytes(self, est_capacity: usize) -> Result<Bytes, Error> {
        let stream = self.content().await?;
        content_to_bytes_inner(stream, est_capacity).await
    }

    pub async fn content_to_bytes_opt(self, est_capacity: usize) -> Result<Option<Bytes>, Error> {
        let Some(stream) = self.content_opt().await? else {
            return Ok(None);
        };

        content_to_bytes_inner(stream, est_capacity).await.map(Some)
    }
}

async fn content_to_bytes_inner(
    stream: ReadStream<impl Stream<Item = Result<Bytes, Error>>>,
    est_capacity: usize,
) -> Result<Bytes, Error> {
    futures::pin_mut!(stream);

    // see if the stream only has 1 chunk, and return it before allocating a (possibly) huge
    // container. Similarly, bail if it's an empty stream.
    let first_chunk = match stream.next().await.transpose()? {
        Some(first_chunk) => first_chunk,
        None => return Ok(Bytes::new()),
    };

    let second_chunk = match stream.next().await.transpose()? {
        Some(second_chunk) => second_chunk,
        None => return Ok(first_chunk),
    };

    let capacity = stream
        .size_hint
        .map(|hint| hint as usize)
        .unwrap_or_else(|| est_capacity.max(first_chunk.len() + second_chunk.len()));

    let mut dst = BytesMut::with_capacity(capacity);

    dst.extend_from_slice(&first_chunk);
    dst.extend_from_slice(&second_chunk);

    // now that we have a buffer, just go until we exhaust the stream.
    while let Some(res) = stream.next().await {
        let chunk = res?;
        dst.extend_from_slice(&chunk);
    }

    Ok(dst.freeze())
}

pin_project_lite::pin_project! {
    pub struct ReadStream<S> {
        headers: HeaderMap,
        size_hint: Option<u64>,
        #[pin]
        stream: S,
    }
}

impl<S> ReadStream<S> {
    pub fn size_hint(&self) -> Option<u64> {
        self.size_hint
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl<S: Stream> Stream for ReadStream<S> {
    type Item = S::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

fn make_range_header<I: PrimInt + Integer>(
    range: impl RangeBounds<I>,
) -> Result<Option<(HeaderValue, Option<u64>)>, InvalidHeaderValue> {
    fn inner<I: PrimInt + Integer>(
        start: Bound<&I>,
        end: Bound<&I>,
    ) -> Result<Option<(HeaderValue, Option<u64>)>, InvalidHeaderValue> {
        let start = match start {
            Bound::Unbounded => None,
            Bound::Included(incl) => Some(*incl),
            Bound::Excluded(excl) => Some(*excl + I::one()),
        };

        let end = match end {
            Bound::Unbounded => None,
            Bound::Included(incl) => Some(*incl),
            Bound::Excluded(excl) => Some(*excl - I::one()),
        };

        macro_rules! fmt_int {
            ($b:expr; $i:expr) => {{
                $b = itoa::Buffer::new();
                $b.format($i)
            }};
        }

        macro_rules! sum_len {
            ($(,)?) => {
                0
            };
            ($first:expr $(,)?) => {
                $first.len()
            };
            ($first:expr $(, $rest:expr)* $(,)?) => {
                $first.len() + sum_len!($($rest,)*)
            };
        }

        macro_rules! make_header {
            ($($part:expr),* $(,)?) => {{
                const PREFIX: &str = "bytes=";
                let mut buf = BytesMut::with_capacity(
                    PREFIX.len() + sum_len!($($part),*),
                );

                buf.extend_from_slice(PREFIX.as_bytes());
                $(
                    buf.extend_from_slice($part.as_bytes());
                )*


                HeaderValue::from_maybe_shared(buf.freeze())
            }};
        }

        let mut b1: itoa::Buffer;
        let mut b2: itoa::Buffer;

        match (start, end) {
            (None, None) => Ok(None),
            (Some(start), None) => {
                // if starting at a negative index to get the last N bytes,
                // we dont need to include a trailing '-'
                let start_str = fmt_int!(b1; start);
                if start < I::zero() {
                    let size = (I::zero() - I::one()) * start;
                    Ok(Some((make_header!(start_str)?, size.to_u64())))
                } else {
                    Ok(Some((make_header!(start_str, "-")?, None)))
                }
            }
            (_, Some(end)) => {
                let start = start.unwrap_or_else(I::zero);
                let size_hint = (end - start).to_u64();

                let start_str = fmt_int!(b1; start);
                let end_str = fmt_int!(b2; end);

                let header = make_header!(start_str, "-", end_str)?;

                Ok(Some((header, size_hint)))
            }
        }
    }

    inner(range.start_bound(), range.end_bound())
}
