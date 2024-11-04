use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::task::JoinHandle;

pin_project_lite::pin_project! {
    pub struct PrefetchStream<I, R> {
        iter: I,
        iter_exhausted: bool,
        loading: VecDeque<JoinHandle<R>>,
        done: VecDeque<Result<R, tokio::task::JoinError>>,
    }
}

impl<I, R> PrefetchStream<I, R> {
    pub fn new<T>(prefetch: usize, iter: T) -> Self
    where
        T: IntoIterator<IntoIter = I>,
        I: Iterator,
        I::Item: FnOnce() -> JoinHandle<R>,
    {
        let mut iter: I = iter.into_iter();

        let mut loading = VecDeque::with_capacity(prefetch);

        loading.extend(iter.by_ref().take(prefetch).map(|init| init()));

        Self {
            iter,
            // the iterator technically could be exhausted here, but the stream poll_next method
            // will set this properly anyways
            iter_exhausted: false,
            loading,
            done: VecDeque::with_capacity(prefetch),
        }
    }

    pub async fn next_item(&mut self) -> Result<Option<R>, tokio::task::JoinError>
    where
        I: Iterator,
        I::Item: FnOnce() -> JoinHandle<R>,
    {
        // try and get an already fetched item first, otherwise results will be out of order
        if let Some(next) = self.done.pop_front() {
            return next.map(Some);
        }

        match self.loading.pop_front() {
            Some(handle) => {
                let ret = handle.await.map(Some);

                // since we popped one from 'loading', we need to insert the next one to
                // not starve prefetched items.
                match self.iter.next() {
                    Some(next) => self.loading.push_back(next()),
                    None => self.iter_exhausted = true,
                }

                ret
            }
            None => Ok(None),
        }
    }

    /// The sum of:
    ///
    ///     - the number of remaining items to fetch
    ///     - the number of items currently being fetched
    ///     - the number of items that have been fetched, but not yet yielded via
    ///       [`Stream::poll_next`] or [`PrefetchStream::next_item`]
    ///
    /// [`Stream::poll_next`]: futures::Stream::poll_next
    pub fn len(&self) -> usize
    where
        I: ExactSizeIterator,
    {
        self.iter.len() + self.loading.len() + self.done.len()
    }

    pub fn is_empty(&self) -> bool
    where
        I: ExactSizeIterator,
    {
        self.len() == 0
    }
}

impl<I, R, F> futures::Stream for PrefetchStream<I, R>
where
    I: Iterator<Item = F>,
    F: FnOnce() -> JoinHandle<R>,
{
    type Item = Result<R, tokio::task::JoinError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        loop {
            match this.loading.front_mut() {
                Some(handle) => match Pin::new(handle).poll(cx) {
                    Poll::Ready(res) => {
                        this.loading.pop_front();
                        // replace the one we just popped (if there's remaning files)
                        if let Some(next) = this.iter.next() {
                            this.loading.push_back(next());
                        } else {
                            *this.iter_exhausted = true;
                        }

                        // push the result and loop to poll the next handle
                        this.done.push_back(res);
                    }
                    Poll::Pending => break,
                },
                None => break,
            }
        }

        match this.done.pop_front() {
            Some(res) => Poll::Ready(Some(res)),
            None if *this.iter_exhausted && this.loading.is_empty() => Poll::Ready(None),
            None => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (remaining_low, remaining_high) = self.iter.size_hint();

        let in_progress_or_done = self.loading.len() + self.done.len();

        let low = remaining_low + in_progress_or_done;
        let high = remaining_high.map(|high| high + in_progress_or_done);

        (low, high)
    }
}
