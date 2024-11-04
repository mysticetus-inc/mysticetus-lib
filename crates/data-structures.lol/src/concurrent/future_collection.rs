
use std::future::Future;
use std::async_iter::AsyncIterator;
use std::task::{Context, Poll};
use std::pin::Pin;




use crate::cursor_vec::CursorVec;

pin_project_lite::pin_project! {
    pub struct FutureCollection<F> {
        futs: CursorVec<Vec<super::Projected<F>>>,
        last_polled: usize,
    }

}


pin_project_lite::pin_project! {
    pub struct FrozenFutureCollection<F> {
        futs: CursorVec<Vec<super::Projected<F>>>,
        last_polled: usize,
    }
}





impl<F> FutureCollection<F> {
    #[inline]
    pub const fn new() -> Self {
        Self { futs: CursorVec(Vec::new()), last_polled: usize::MAX }
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self { futs: CursorVec(Vec::with_capacity(cap)), last_polled: usize::MAX }
    }

    #[inline]
    pub fn push(&mut self, fut: F) {
        self.futs.push(super::Projected::new(fut));
    }

    #[inline]
    pub fn freeze(self) -> FrozenFutureCollection<F> {
        FrozenFutureCollection { futs: self.futs, last_polled: self.last_polled }
    }
}

impl<F> FromIterator<F> for FutureCollection<F> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = F>
    {
        let futs = CursorVec(Vec::from_iter(iter.into_iter().map(super::Projected::new)));

        Self { futs, last_polled: usize::MAX }
    }
}

impl<F> Extend<F> for FutureCollection<F> {
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = F>,
    {
        self.futs.extend(iter.into_iter().map(super::Projected::new));
    }
}


fn poll_inner<F>(
    futs: &mut CursorVec<Vec<super::Projected<F>>>,
    last_polled: &mut usize,
    cx: &mut Context<'_>,
) -> Option<F::Output>
where
    F: Future
{
    if futs.is_empty() {
        return None;
    }

    let len = futs.len();
    let start_at = last_polled.wrapping_add(1) % len;

    let mut cursor = futs.cursor();
    cursor.move_to_index(start_at);

    for _ in 0..len {
        let fut = match cursor.current_mut() {
            Some(fut) => fut,
            None => break,
        };

        if let Poll::Ready(item) = fut.poll_fut(cx) {
            cursor.remove();
            *last_polled = cursor.index();
            return Some(item);
        }
    }

    *last_polled = cursor.index();

    None
}


impl<F> AsyncIterator for FutureCollection<F>
where
    F: Future
{
    type Item = F::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match poll_inner(this.futs, this.last_polled, cx) {
            Some(item) => Poll::Ready(Some(item)),
            None => Poll::Pending,
        }
    }
}
