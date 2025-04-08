use std::pin::Pin;

use bytes::{BufMut, Bytes, BytesMut};
use http_body::Body;
use hyper::body::Incoming;

pub(crate) async fn collect_body(body: Incoming) -> Result<Bytes, hyper::Error> {
    async fn wait_for_data(body: &mut Pin<&mut Incoming>) -> Result<Option<Bytes>, hyper::Error> {
        loop {
            match std::future::poll_fn(|cx| body.as_mut().poll_frame(cx)).await {
                None => return Ok(None),
                Some(result) => match result?.into_data() {
                    Ok(data) => return Ok(Some(data)),
                    Err(_trailers) => continue,
                },
            }
        }
    }

    let mut body = std::pin::pin!(body);

    let Some(first_chunk) = wait_for_data(&mut body).await? else {
        // don't error on an empty body, let the caller handle that.
        return Ok(Bytes::new());
    };

    let second_chunk = match wait_for_data(&mut body).await? {
        // if we only get one chunk, return it so we dont need to allocate more
        None => return Ok(first_chunk),
        Some(second_chunk) => second_chunk,
    };

    // if we didnt return, we need to concatenate buffers
    // and continue polling for the rest of the data
    let mut dst = match first_chunk.try_into_mut() {
        Ok(dst) => dst,
        Err(bytes) => {
            let mut dst = BytesMut::with_capacity(bytes.len() + second_chunk.len());
            dst.put(bytes);
            dst
        }
    };

    dst.put(second_chunk);

    while let Some(next_chunk) = wait_for_data(&mut body).await? {
        dst.put(next_chunk);
    }

    Ok(dst.freeze())
}
