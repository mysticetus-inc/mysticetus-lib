//! A type that provides a way to return 1 of something in a [`Future`] or [`Stream`].
//!
//! [`Future`]: [`std::future::Stream`]
//! [`Stream`]: [`futures::Stream`]

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;

pin_project_lite::pin_project! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct Once<T> {
        inner: Option<T>
    }
}

impl<T> Once<T> {
    pub fn new(item: T) -> Self {
        Self { inner: Some(item) }
    }

    pub fn into_inner(self) -> Option<T> {
        self.inner
    }

    pub fn get(&self) -> Option<&T> {
        self.inner.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.as_mut()
    }

    pub fn has_yielded(&self) -> bool {
        self.inner.is_none()
    }
}

impl<T> Default for Once<T>
where
    T: Default,
{
    fn default() -> Self {
        Once::new(T::default())
    }
}

impl<T> From<T> for Once<T> {
    fn from(item: T) -> Self {
        Self::new(item)
    }
}

impl<T> Future for Once<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = self
            .project()
            .inner
            .take()
            .expect("Once<T> polled after completion");

        Poll::Ready(item)
    }
}

impl<T> Stream for Once<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().inner.take())
    }
}
