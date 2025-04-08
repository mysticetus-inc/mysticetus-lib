use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::buf::Writer;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{Stream, TryStream};
use serde::ser::{Serialize, SerializeSeq};

pin_project_lite::pin_project! {
    pub struct JsonStream<S> {
        #[pin]
        state: State<S>,
        serializer: serde_json::Serializer<Writer<BytesMut>>,
    }
}

pin_project_lite::pin_project! {
    #[project = StateProjection]
    enum State<S> {
        Streaming { #[pin] stream: S },
        Done,
        Errored,
    }
}

impl<S> JsonStream<S> {
    pub fn new(stream: S, capacity: usize) -> Self {
        Self {
            state: State::Streaming { stream },
            serializer: serde_json::Serializer::new(BytesMut::with_capacity(capacity).writer()),
        }
    }
}

impl<S> axum::response::IntoResponse for JsonStream<S>
where
    S: Send + Stream<Item = Result<S::Ok, S::Error>> + TryStream,
    S::Ok: IntoIterator<Item: serde::Serialize>,
    S::Error: Into<()>,
{
    fn into_response(self) -> axum::response::Response {
        (
            (http::header::CONTENT_TYPE, "application/json"),
            axum::body::Body::from_stream(self),
        )
            .into_response()
    }
}

impl<S> Stream for JsonStream<S>
where
    S: Stream<Item = Result<S::Ok, S::Error>> + TryStream,
    S::Ok: IntoIterator<Item: serde::Serialize>,
    S::Error: axum::response::IntoResponse,
{
    type Item = Result<Bytes, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}
