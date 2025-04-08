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
    #[repr(transparent)]
    pub struct RequestSink<T> {
        // since we can't close the channel from the sender side, we just drop the
        // sender instead. Since RequestSink isn't 'Clone', only 1 sender
        // ever exists, which will cause the reciever to hang up when dropped.
        tx: Option<UnboundedSender<T>>,
    }
}

impl<T> RequestSink<T> {
    #[inline]
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

    pub fn forward_stream(
        &self,
        stream: impl Stream<Item = T>,
    ) -> ForwardStream<'_, impl Stream<Item = T>> {
        ForwardStream { sink: self, stream }
    }

    #[inline]
    pub fn close(&mut self) {
        self.tx = None;
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        match self.tx {
            Some(ref tx) => tx.is_closed(),
            None => true,
        }
    }
}

impl<T> futures::Sink<T> for RequestSink<T> {
    type Error = SendError<T>;

    #[inline]
    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().send(item)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().tx = None;
        Poll::Ready(Ok(()))
    }
}

pin_project_lite::pin_project! {
    #[repr(transparent)]
    pub struct RequestStream<T> {
        rx: UnboundedReceiver<T>,
    }
}

impl<T> Stream for RequestStream<T> {
    type Item = T;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().rx.poll_recv(cx)
    }
}

pin_project_lite::pin_project! {
    pub struct ForwardStream<'a, S: Stream> {
        #[pin]
        stream: S,
        sink: &'a RequestSink<S::Item>,
    }
}

impl<S> Future for ForwardStream<'_, S>
where
    S: Stream,
{
    type Output = Result<(), SendError<S::Item>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        while let Some(item) = std::task::ready!(this.stream.as_mut().poll_next(cx)) {
            this.sink.send(item)?;
        }

        Poll::Ready(Ok(()))
    }
}
