pub mod future_collection;

use std::future::Future;
use std::async_iter::AsyncIterator;
use std::pin::Pin;
use std::task::{Context, Poll};


pin_project_lite::pin_project! {
    pub(crate) struct Projected<T> {
        #[pin]
        inner: T,
    }
}

impl<T> Projected<T> {
    #[inline]
    pub const fn new(item: T) -> Self {
        Self { inner: item }
    }
}

impl<T> const std::ops::Deref for Projected<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> const std::ops::DerefMut for Projected<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> const AsRef<T> for Projected<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.inner
    }
}


impl<T> const AsMut<T> for Projected<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> const From<T> for Projected<T> {
    #[inline]
    fn from(item: T) -> Self {
        Self::new(item)
    }
}

impl<T> Future for Projected<T>
where
    T: Future
{
    type Output = T::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<T> AsyncIterator for Projected<T>
where
    T: AsyncIterator
{
    type Item = T::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}
