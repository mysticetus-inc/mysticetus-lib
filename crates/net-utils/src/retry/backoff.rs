use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::task::{Context, Poll, ready};

use tokio::time::Sleep;
use tower::retry::Policy;

use crate::backoff::Backoff;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Classify {
    Retry,
    DontRetry,
}

pub trait RetryClassify<Req, Resp, E>: Clone {
    fn classify(&self, req: &Req, result: Result<&Resp, &E>) -> Classify;
}

impl<Req, Resp, E, F> RetryClassify<Req, Resp, E> for F
where
    F: Fn(&Req, Result<&Resp, &E>) -> Classify + Clone,
{
    fn classify(&self, req: &Req, result: Result<&Resp, &E>) -> Classify {
        (self)(req, result)
    }
}

#[derive(Debug, Clone)]
pub struct RetryBackoffPolicy<E> {
    classifier: E,
    backoff: Backoff<Arc<AtomicU32>>,
}

impl<Req, Resp, E, C> Policy<Req, Resp, E> for RetryBackoffPolicy<C>
where
    C: RetryClassify<Req, Resp, E>,
    Req: Clone,
{
    type Future = RetryBackoffFuture<C>;

    fn retry(&self, req: &Req, result: Result<&Resp, &E>) -> Option<Self::Future> {
        if self.classifier.classify(req, result) == Classify::DontRetry {
            return None;
        }

        let backoff_once = self.backoff.backoff_once_ref()?;

        let sleep = backoff_once.into_future();

        Some(RetryBackoffFuture {
            sleep,
            policy: Some(self.clone()),
        })
    }

    fn clone_request(&self, req: &Req) -> Option<Req> {
        Some(Req::clone(req))
    }
}

pin_project_lite::pin_project! {
    pub struct RetryBackoffFuture<C> {
        #[pin]
        sleep: Sleep,
        policy: Option<RetryBackoffPolicy<C>>,
    }
}

impl<C> Future for RetryBackoffFuture<C> {
    type Output = RetryBackoffPolicy<C>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        ready!(this.sleep.poll(cx));

        let policy = this
            .policy
            .take()
            .expect("RetryBackoffFuture polled after completion");

        Poll::Ready(policy)
    }
}
