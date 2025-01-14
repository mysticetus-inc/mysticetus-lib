use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, TryFuture, TryStream};
use tokio::sync::oneshot;

pub fn into_infallible<S: TryStream>(stream: S) -> (Handle<S>, Driver<S>) {
    let (sender, recv) = oneshot::channel();

    let driver = Driver {
        state: State::Stream {
            stream,
            sender: Some(sender),
        },
    };

    let handle = Handle { recv };

    (handle, driver)
}

pin_project_lite::pin_project! {
    pub struct Driver<S: TryStream> {
        #[pin]
        state: State<S>,
    }
}

pin_project_lite::pin_project! {
    pub struct Handle<S: TryStream> {
        #[pin]
        recv: oneshot::Receiver<S::Error>,
    }
}

impl<S: TryStream> Handle<S> {
    pub async fn race_with<F, Error>(self, future: F) -> Result<F::Ok, Error>
    where
        F: TryFuture + Future<Output = Result<F::Ok, F::Error>>,
        Error: From<S::Error> + From<F::Error>,
    {
        let future = std::pin::pin!(future);

        match futures::future::select(self, future).await {
            futures::future::Either::Left((result, future)) => {
                result?;
                future.await.map_err(Error::from)
            }
            futures::future::Either::Right((result, future)) => {
                let item = result?;
                future.await?;
                Ok(item)
            }
        }
    }
}

impl<S: TryStream> Future for Handle<S> {
    type Output = Result<(), S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match std::task::ready!(self.project().recv.poll(cx)) {
            Ok(error) => Poll::Ready(Err(error)),
            // getting a reciever error means the sender was dropped without
            // needing to pass along any errors.
            Err(_recv) => Poll::Ready(Ok(())),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = StateProjection]
    enum State<S: TryStream> {
        Stream {
            #[pin]
            stream: S,
            sender: Option<oneshot::Sender<S::Error>>,
        },
        Done,
    }
}

impl<S> Stream for Driver<S>
where
    S: TryStream + Stream<Item = Result<S::Ok, S::Error>>,
{
    type Item = S::Ok;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.state.as_mut().project() {
            StateProjection::Done => Poll::Ready(None),
            StateProjection::Stream { stream, sender } => {
                match std::task::ready!(stream.poll_next(cx)) {
                    Some(Ok(item)) => Poll::Ready(Some(item)),
                    Some(Err(error)) => {
                        let sender = sender.take().expect("invalid state");
                        this.state.set(State::Done);
                        let _ = sender.send(error);
                        Poll::Ready(None)
                    }
                    None => {
                        this.state.set(State::Done);
                        Poll::Ready(None)
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            State::Done => (0, Some(0)),
            State::Stream { ref stream, .. } => (0, stream.size_hint().1),
        }
    }
}
