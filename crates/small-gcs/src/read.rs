use std::task::Poll;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{Stream, StreamExt};
use net_utils::backoff::Backoff;
use reqwest::{header, RequestBuilder};

use crate::client::Client;
use crate::params::Alt;
use crate::{Error, Object};

pub struct ReadBuilder<'a> {
    builder: RequestBuilder,
    shared: &'a Client,
}

impl<'a> ReadBuilder<'a> {
    #[inline]
    pub(super) fn new(shared: &'a Client, bucket: &str, path: &str) -> Self {
        let url = crate::url::UrlBuilder::new(bucket).name(path).format();

        ReadBuilder {
            builder: shared.client.get(url),
            shared,
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
            shared,
        }
    }

    #[inline]
    fn with_query_param<S>(self, field: &str, value: S) -> Self
    where
        S: serde::Serialize,
    {
        Self {
            builder: self.builder.query(&[(field, value)]),
            shared: self.shared,
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

    async fn send(self, alt: Alt) -> Result<reqwest::Response, Error> {
        let auth_header = self.shared.auth.get_header().await?;

        let request = self
            .builder
            .header(header::AUTHORIZATION, auth_header)
            .query(&[alt])
            .build()?;

        let resp =
            crate::try_execute_with_backoff(&self.shared.client, request, Backoff::default).await?;

        crate::validate_response(resp).await
    }

    pub async fn metadata(self) -> Result<Object, Error> {
        let resp = self
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

    pub async fn content(self) -> Result<impl Stream<Item = Result<Bytes, Error>>, Error> {
        use futures::TryStreamExt;

        let resp = self.send(Alt::Media).await?;

        Ok(resp.bytes_stream().map_err(Error::Reqwest))
    }

    pub async fn content_opt(
        self,
    ) -> Result<Option<impl Stream<Item = Result<Bytes, Error>>>, Error> {
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

        let mut chunks = std::collections::LinkedList::new();
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

            // write whatever chunks we have to the writer
            let mut cursor = chunks.cursor_front_mut();

            // do this in a loop so we can write multiple chunks if the writer can handle it.
            loop {
                // get the current front chunk, if there isnt one, return if the stream is finished,
                // since that means we're done. otherwise, return pending since we're waiting
                // on the stream.
                let chunk = match cursor.current() {
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
                                cursor.remove_current();
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
    stream: impl Stream<Item = Result<Bytes, Error>>,
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

    let mut dst = BytesMut::with_capacity(est_capacity.max(first_chunk.len() + second_chunk.len()));

    dst.extend_from_slice(&first_chunk);
    dst.extend_from_slice(&second_chunk);

    // now that we have a buffer, just go until we exhaust the stream.
    while let Some(res) = stream.next().await {
        let chunk = res?;
        dst.extend_from_slice(&chunk);
    }

    Ok(dst.freeze())
}
