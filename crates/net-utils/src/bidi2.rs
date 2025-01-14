use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub fn build_pair<T>() -> (RequestSink<T>, RequestStream<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (RequestSink { tx: Some(tx) }, RequestStream { rx })
}

pin_project_lite::pin_project! {
    pub struct RequestSink<T> {
        tx: Option<UnboundedSender<T>>,
    }
}

impl<T> RequestSink<T> {
    pub fn send(&self, item: T) -> Result<(), SendError<T>> {
        match self.tx {
            Some(ref tx) => tx.send(item),
            None => Err(SendError(item)),
        }
    }

    pub fn send_all(&self, iter: &mut impl Iterator<Item = T>) -> Result<(), SendError<T>> {
        for item in iter {
            self.send(item)?;
        }
        Ok(())
    }

    pub async fn forward_stream(&self, stream: impl Stream<Item = T>) -> Result<(), SendError<T>> {
        ForwardStream { sink: self, stream }.await
    }

    pub fn close(&mut self) {
        self.tx = None;
    }

    pub fn is_closed(&self) -> bool {
        match self.tx {
            Some(ref tx) => tx.is_closed(),
            None => true,
        }
    }
}

impl<T> futures::Sink<T> for RequestSink<T> {
    type Error = SendError<T>;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().tx = None;
        Poll::Ready(Ok(()))
    }
}

pin_project_lite::pin_project! {
    pub struct RequestStream<T> {
        rx: UnboundedReceiver<T>,
    }
}

impl<T> Stream for RequestStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().rx.poll_recv(cx)
    }
}

pin_project_lite::pin_project! {
    pub struct ForwardStream<'a, S, T> {
        #[pin]
        stream: S,
        sink: &'a RequestSink<T>,
    }
}

impl<S, T> Future for ForwardStream<'_, S, T>
where
    S: Stream<Item = T>,
{
    type Output = Result<(), SendError<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        while let Some(item) = std::task::ready!(this.stream.as_mut().poll_next(cx)) {
            this.sink.send(item)?;
        }

        Poll::Ready(Ok(()))
    }
}
