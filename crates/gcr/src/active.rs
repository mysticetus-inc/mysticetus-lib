//! [`Active`], a [`Layer`] that tracks the number of currently active requests.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};

use tower_layer::Layer;
use tower_service::Service;

static ACTIVE: AtomicUsize = AtomicUsize::new(0);

/// A [`Layer`] that keeps track of the number of currently active requests.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Active;

impl Active {
    #[inline]
    pub fn current() -> usize {
        ACTIVE.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn current_with_order(order: Ordering) -> usize {
        ACTIVE.load(order)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActiveService<S> {
    inner: S,
}

impl<S> Layer<S> for Active {
    type Service = ActiveService<S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        ActiveService { inner }
    }
}

impl<R, S> Service<R> for ActiveService<S>
where
    S: Service<R>,
{
    type Error = S::Error;
    type Response = S::Response;
    type Future = ActiveFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: R) -> Self::Future {
        ACTIVE.fetch_add(1, Ordering::Relaxed);
        ActiveFuture(self.inner.call(req))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ActiveFuture<F>(F);

impl<F: Future> Future for ActiveFuture<F> {
    type Output = F::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: (likely) safe, because:
        //  - No pinning invariants are broken, as nothing is moved/no references are invalidated
        //  - ActiveFuture is repr(transparent), so transmuting a mutable reference to it is safe.
        //  - Pin is also repr(transparent), so transmuting it should be safe too.
        //  - ActiveFuture has no Drop impl, so no weird drop side effects can happen.
        unsafe {
            match std::mem::transmute::<Pin<&mut Self>, Pin<&mut F>>(self).poll(cx) {
                Poll::Ready(output) => {
                    ACTIVE.fetch_sub(1, Ordering::Relaxed);
                    Poll::Ready(output)
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
