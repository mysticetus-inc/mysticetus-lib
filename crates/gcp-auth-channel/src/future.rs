use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::TryFuture;

pin_project_lite::pin_project! {
    #[project = JoinFutureProjection]
    pub struct JoinFuture<Fut1: TryFuture, Fut2: TryFuture> {
        #[pin]
        fut1: State<Fut1>,
        #[pin]
        fut2: State<Fut2>,
    }
}

pin_project_lite::pin_project! {
    #[project = StateProjection]
    enum State<F: TryFuture> {
        Pending { #[pin] fut: F },
        Ok { ok: Option<F::Ok> },
    }
}

impl<Fut1, Fut2> Future for JoinFuture<Fut1, Fut2>
where
    Fut1: TryFuture + Future<Output = Result<Fut1::Ok, Fut1::Error>>,
    Fut2: TryFuture + Future<Output = Result<Fut2::Ok, Fut2::Error>>,
    Fut2::Error: Into<Fut1::Error>,
{
    type Output = Result<(Fut1::Ok, Fut2::Ok), Fut1::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use StateProjection::*;

        let JoinFutureProjection { mut fut1, mut fut2 } = self.project();

        match (fut1.as_mut().project(), fut2.as_mut().project()) {
            (Ok { ok: ok1 }, Ok { ok: ok2 }) => {
                let first = ok1.take().expect("JoinFuture polled after completion");
                let second = ok2.take().expect("JoinFuture polled after completion");
                Poll::Ready(Ok((first, second)))
            }
            (Pending { fut: future1 }, Pending { fut: future2 }) => {
                match (future1.poll(cx), future2.poll(cx)) {
                    (Poll::Pending, Poll::Pending) => Poll::Pending,
                    (Poll::Ready(Err(error)), _) => Poll::Ready(Err(error)),
                    (_, Poll::Ready(Err(error))) => Poll::Ready(Err(error.into())),
                    (Poll::Ready(Ok(first)), Poll::Pending) => {
                        fut1.set(State::Ok { ok: Some(first) });
                        Poll::Pending
                    }
                    (Poll::Pending, Poll::Ready(Ok(second))) => {
                        fut2.set(State::Ok { ok: Some(second) });
                        Poll::Pending
                    }
                    (Poll::Ready(Ok(first)), Poll::Ready(Ok(second))) => {
                        fut1.set(State::Ok { ok: None });
                        fut2.set(State::Ok { ok: None });
                        Poll::Ready(Ok((first, second)))
                    }
                }
            }
        }
    }
}
