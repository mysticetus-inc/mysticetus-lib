use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::Stream;
use pin_project_lite::pin_project;
use tokio::sync::oneshot;

use crate::bidirec::DecomposeResponse;

pin_project! {
    /// A wrapper around a bidirectional gRPC, but unlike [`Bidirec`], this is optimized for
    /// situations where only 2 requests will be sent in the stream, an opening message and a
    /// closing message.
    pub struct OpenClose<Req, Resp> {
        tx: Option<oneshot::Sender<Req>>,
        #[pin]
        handle: tonic::Streaming<Resp>,
    }
}

impl<Req, Resp> fmt::Debug for OpenClose<Req, Resp> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenClose")
            .field("is_closed", &self.is_closed())
            .field("handle", &self.handle)
            .finish()
    }
}

impl<Req, Resp> OpenClose<Req, Resp> {
    /// Sends the closing request. If a prior closing request was already sent, returns the
    /// request as an [`Err`]. Will also return [`Err`] if [`tonic`] dropped the request handle,
    /// but ideally that should never happen.
    pub fn close(&mut self, req: Req) -> Result<(), Req> {
        if let Some(tx) = self.tx.take() {
            tx.send(req)
        } else {
            Err(req)
        }
    }

    /// Identical to [`tonic::Streaming::message`].
    pub async fn message(&mut self) -> Result<Option<Resp>, tonic::Status> {
        self.handle.message().await
    }

    /// Checks if the close message has already been sent. If `tonic` dropped the request stream
    /// this will also return true if a message hasn't been sent, but it ideally should never do
    /// that.
    pub fn is_closed(&self) -> bool {
        match self.tx.as_ref() {
            Some(tx) => tx.is_closed(),
            None => true,
        }
    }
}

impl<Req, Resp> Stream for OpenClose<Req, Resp> {
    type Item = Result<Resp, tonic::Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().handle.poll_next(cx)
    }
}

/// Returns the parts needed to construct an [`OpenClose`] handle.
pub fn build_parts<Req>(open: Req) -> (RequestStream<Req>, PartiallyInit<Req>) {
    let (tx, close) = oneshot::channel();

    (
        RequestStream {
            open: Some(open),
            close,
            closed: false,
        },
        PartiallyInit { tx },
    )
}

pub struct PartiallyInit<Req> {
    tx: oneshot::Sender<Req>,
}

impl<Req> PartiallyInit<Req> {
    pub async fn try_initialize<Fut, R, E, Resp>(self, fut: Fut) -> Result<OpenClose<Req, Resp>, E>
    where
        Fut: Future<Output = Result<R, E>>,
        R: DecomposeResponse<tonic::Streaming<Resp>>,
    {
        let handle = fut.await?.decompose();
        Ok(OpenClose {
            handle,
            tx: Some(self.tx),
        })
    }
}

pin_project! {
    pub struct RequestStream<Req> {
        open: Option<Req>,
        #[pin]
        close: oneshot::Receiver<Req>,
        closed: bool,
    }
}

impl<Req> Stream for RequestStream<Req> {
    type Item = Req;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if let Some(open) = this.open.take() {
            return Poll::Ready(Some(open));
        } else if *this.closed {
            // need to guard polling 'close' after it yields an item, it panics if so.
            return Poll::Ready(None);
        }

        let resp = ready!(this.close.poll(cx));

        *this.closed = true;

        match resp {
            Ok(close) => Poll::Ready(Some(close)),
            // Err can only happen if the sender is dropped before sending a value.
            // in that case, just return None since nothing else can be yielded.
            Err(_) => Poll::Ready(None),
        }
    }
}
