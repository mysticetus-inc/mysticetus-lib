//! [`Bidirec`], a handle to a bidirectional gRPC stream.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::{Context, Poll, ready};

use futures::{Future, Stream};
use pin_project_lite::pin_project;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
struct StreamState {
    pending: AtomicUsize,
    closed: AtomicBool,
}

pin_project! {
    #[derive(Debug)]
    pub struct Bidirec<Req, Resp> {
        req_tx: UnboundedSender<Req>,
        state: Arc<StreamState>,
        #[pin]
        result_stream: tonic::Streaming<Resp>,
    }
}

#[derive(Debug)]
pub struct PartiallyInit<Req> {
    req_tx: UnboundedSender<Req>,
    state: Arc<StreamState>,
}

pub fn build_parts<Req>() -> (RequestStream<Req>, PartiallyInit<Req>) {
    let (req_tx, req_rx) = mpsc::unbounded_channel();

    let state = Arc::new(StreamState {
        pending: AtomicUsize::new(0),
        closed: AtomicBool::new(false),
    });

    let req_stream = RequestStream {
        req_rx,
        state: Arc::clone(&state),
        rx_closed: false,
    };

    (req_stream, PartiallyInit { req_tx, state })
}

impl<Req> Bidirec<Req, ()> {
    /// Identical to [`build_parts`], but can be used without an extra import/etc.
    #[inline]
    pub fn parts() -> (RequestStream<Req>, PartiallyInit<Req>) {
        build_parts()
    }
}

/// Trait that allows the [`PartiallyInit::initialize`] and [`try_initialize`] functions
/// to handle both [`tonic::Response<T>`] as well as the inner [`T`] if
/// [`tonic::Response::into_inner`] has already been called.
///
/// [`try_initialize`]: [`PartiallyInit::try_initialize`]
pub trait DecomposeResponse<T> {
    fn decompose(self) -> T;
}

impl<T> DecomposeResponse<T> for tonic::Response<T> {
    fn decompose(self) -> T {
        self.into_inner()
    }
}

impl<T> DecomposeResponse<T> for T {
    fn decompose(self) -> T {
        self
    }
}

impl<Req> PartiallyInit<Req> {
    pub async fn initialize<Fut, R, Resp>(self, fut: Fut) -> Bidirec<Req, Resp>
    where
        Fut: Future<Output = R>,
        R: DecomposeResponse<tonic::Streaming<Resp>>,
    {
        Bidirec {
            req_tx: self.req_tx,
            state: self.state,
            result_stream: fut.await.decompose(),
        }
    }

    pub async fn try_initialize<Fut, R, E, Resp>(self, fut: Fut) -> Result<Bidirec<Req, Resp>, E>
    where
        Fut: Future<Output = Result<R, E>>,
        R: DecomposeResponse<tonic::Streaming<Resp>>,
    {
        let result_stream = fut.await?.decompose();

        Ok(Bidirec {
            req_tx: self.req_tx,
            state: self.state,
            result_stream,
        })
    }
}

impl<Req, Resp> Bidirec<Req, Resp> {
    pub async fn init<F, Fut, E>(init_stream_fn: F) -> Result<Self, E>
    where
        F: FnOnce(RequestStream<Req>) -> Fut,
        Fut: futures::Future<Output = Result<tonic::Streaming<Resp>, E>>,
    {
        let (req, partial) = build_parts();
        partial.try_initialize(init_stream_fn(req)).await
    }

    pub async fn message(&mut self) -> Result<Option<Resp>, tonic::Status> {
        if let Some(resp) = self.result_stream.message().await? {
            self.state.pending.fetch_sub(1, Ordering::SeqCst);
            Ok(Some(resp))
        } else {
            Ok(None)
        }
    }

    pub fn pending_messages(&self) -> usize {
        self.state.pending.load(Ordering::SeqCst)
    }

    pub fn is_closed(&self) -> bool {
        self.req_tx.is_closed() || self.state.closed.load(Ordering::SeqCst)
    }

    pub fn send(&self, req: Req) -> Result<(), Req> {
        if !self.is_closed() {
            self.req_tx.send(req).map_err(|err| err.0)
        } else {
            Err(req)
        }
    }

    pub fn close(&mut self) {
        self.state.closed.store(true, Ordering::SeqCst)
    }
}

impl<Req, Resp> Stream for Bidirec<Req, Resp> {
    type Item = Result<Resp, tonic::Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().result_stream.poll_next(cx)
    }
}

pub struct RequestStream<Req> {
    req_rx: UnboundedReceiver<Req>,
    state: Arc<StreamState>,
    rx_closed: bool,
}

impl<Req> Stream for RequestStream<Req> {
    type Item = Req;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // if we're not locally indicated as having closed, but the shared state is marked as
        // closed, actually close the channel here (since only the recieving end can do that).
        if !this.rx_closed && this.state.closed.load(Ordering::SeqCst) {
            this.req_rx.close();
            this.rx_closed = true;
        }

        match ready!(this.req_rx.poll_recv(cx)) {
            Some(req) => {
                // if we return a request, increment the pending responses.
                this.state.pending.fetch_add(1, Ordering::SeqCst);
                Poll::Ready(Some(req))
            }
            None => Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let pending = self.state.pending.load(Ordering::SeqCst);
        (pending, Some(pending))
    }
}
