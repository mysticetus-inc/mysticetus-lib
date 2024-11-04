//! [`Timed`]/[`MaybeTimed`] traits, plus iterator utilities for types with [`Timed`] items.
use std::collections::VecDeque;

use super::{Duration, Timestamp};

/// A trait that's implemented by something containing a [`Timestamp`].
pub trait Timed {
    /// Returns the timestamp represented by this item.
    fn timestamp(&self) -> Timestamp;
}

/// A trait that's implemented by something that may have an internal [`Timestamp`]. Anything
/// that implements [`Timed`] also gets this for free.
pub trait MaybeTimed {
    /// Returns a possible timestamp represented by this item.
    fn opt_timestamp(&self) -> Option<Timestamp>;
}

impl Timed for Timestamp {
    fn timestamp(&self) -> Timestamp {
        *self
    }
}

impl<T> Timed for &'_ T
where
    T: Timed,
{
    fn timestamp(&self) -> Timestamp {
        T::timestamp(self)
    }
}

impl<T> MaybeTimed for T
where
    T: Timed,
{
    fn opt_timestamp(&self) -> Option<Timestamp> {
        Some(self.timestamp())
    }
}

/// [`Iterator`] extension trait, adding [`Timed`]/[`MaybeTimed`] utilities to any existing
/// [`Iterator`].
pub trait TimedIterExt: Iterator {
    /// Provides an iterator that skips all elements that are out of monotonic order.
    fn monotonic(&mut self) -> MonotonicIter<&mut Self>
    where
        Self::Item: Timed,
    {
        MonotonicIter::new(self)
    }

    /// Provides an iterator that skips all elements that are either missing timestamps, or are
    /// out of monotonic order.
    fn maybe_monotonic(&mut self) -> MaybeMonotonicIter<&mut Self>
    where
        Self::Item: MaybeTimed,
    {
        MaybeMonotonicIter::new(self)
    }

    /// Consumes the iterator, returning true if the remaning elements are in monotonic order.
    /// Also returns 'true' if the iterator is empty and yields no elements.
    #[allow(clippy::wrong_self_convention)]
    // taking '&mut' would be a weird interface for something that consumes the iterator.
    fn is_monotonic(mut self) -> bool
    where
        Self: Sized,
        Self::Item: MaybeTimed,
    {
        let mut last_ts = loop {
            match self.next() {
                Some(item) => {
                    if let Some(ts) = item.opt_timestamp() {
                        break ts;
                    }
                }
                None => return true,
            }
        };

        for item in self {
            if let Some(ts) = item.opt_timestamp() {
                if ts < last_ts {
                    return false;
                } else {
                    last_ts = ts;
                }
            }
        }

        true
    }

    /// Iterates over items monotonically, skipping elements in order to get the gap between
    /// items to optimize `|current_ts - last_ts| = step_duration`. Since elements may have large
    /// gaps in time, this adapter yields a tuple of the duration since the last step, and the
    /// item itself.
    fn step_duration<D>(&mut self, step_duration: D) -> StepDuration<&mut Self>
    where
        D: Into<Duration>,
        Self::Item: Timed,
    {
        StepDuration::new(self, step_duration)
    }
}

impl<T> TimedIterExt for T where T: Iterator {}

/// A wrapper over an inner iterator of [`Timed`] items, only yielding items who's
/// [`Timestamp`]s are increasing in monotonic order.
#[derive(Debug, Clone)]
pub struct MonotonicIter<I> {
    inner: I,
    last_ts: Option<Timestamp>,
}

impl<I> MonotonicIter<I>
where
    I: Iterator,
    I::Item: Timed,
{
    /// Builds a new monotonic iterator.
    pub fn new<A>(iter: A) -> Self
    where
        A: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: iter.into_iter(),
            last_ts: None,
        }
    }
}

impl<I> Iterator for MonotonicIter<I>
where
    I: Iterator,
    I::Item: Timed,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! set_last_ts {
            ($item:expr) => {{
                self.last_ts = Some($item.timestamp());
                Some($item)
            }};
        }

        let last_ts = match self.last_ts {
            Some(ts) => ts,
            None => return set_last_ts!(self.inner.next()?),
        };

        for item in self.inner.by_ref() {
            if item.timestamp() > last_ts {
                return set_last_ts!(item);
            }
        }

        None
    }
}

/// A wrapper over an inner iterator of [`MaybeTimed`] items, only yielding items who's
/// [`Timestamp`]s both exist, and in increasing monotonic order.
#[derive(Debug, Clone)]
pub struct MaybeMonotonicIter<I> {
    inner: I,
    last_ts: Option<Timestamp>,
}

impl<I> MaybeMonotonicIter<I>
where
    I: Iterator,
    I::Item: MaybeTimed,
{
    /// Builds a new monotonic iterator.
    pub fn new<A>(iter: A) -> Self
    where
        A: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: iter.into_iter(),
            last_ts: None,
        }
    }
}

impl<I> Iterator for MaybeMonotonicIter<I>
where
    I: Iterator,
    I::Item: MaybeTimed,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let last_ts = match self.last_ts {
            Some(ts) => ts,
            None => {
                for item in self.inner.by_ref() {
                    if let Some(ts) = item.opt_timestamp() {
                        self.last_ts = Some(ts);
                        return Some(item);
                    }
                }

                return None;
            }
        };

        for item in self.inner.by_ref() {
            if let Some(item_ts) = item.opt_timestamp() {
                if item_ts > last_ts {
                    self.last_ts = Some(item_ts);
                    return Some(item);
                }
            }
        }

        None
    }
}

/// An [`Iterator`] adapter that attempts to step over elements (monotonically) in even(ish) sized
/// steps in duration. Since exact sized steps are unlikely, the item returned is a tuple pair,
/// where the first element is actual duration since the last step, and the 2nd element is the
/// item itself. The first delta/duration yielded by this iterator will be
/// [`time::Duration::ZERO`].
pub struct StepDuration<I>
where
    I: Iterator,
{
    iter: MonotonicIter<I>,
    step_duration: Duration,
    last_item_ts: Option<Timestamp>,
    buffer: VecDeque<(Duration, I::Item)>,
}

impl<I> StepDuration<I>
where
    I: Iterator,
    I::Item: Timed,
{
    /// Builds a new step duration iterator, with the given step size.
    pub fn new<A, D>(iter: A, step_duration: D) -> Self
    where
        A: IntoIterator<IntoIter = I>,
        D: Into<Duration>,
    {
        Self {
            iter: MonotonicIter::new(iter),
            step_duration: step_duration.into(),
            last_item_ts: None,
            buffer: VecDeque::new(),
        }
    }
}

impl<I> Iterator for StepDuration<I>
where
    I: Iterator,
    I::Item: Timed,
{
    type Item = (Duration, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let last_ts = match self.last_item_ts {
            Some(ts) => ts,
            None => {
                let item = self.iter.next()?;
                self.last_item_ts = Some(item.timestamp());
                return Some((Duration::ZERO, item));
            }
        };

        // then, update all remaining elements so that the duration is relative to 'last_ts'
        self.buffer
            .iter_mut()
            .for_each(|(delta, item)| *delta = item.timestamp() - last_ts);

        // get the mininum duration in the buffer, or fallback on
        let mut min_delta = self
            .buffer
            .iter()
            .map(|(dur, _)| (self.step_duration - *dur).abs())
            .min()
            .unwrap_or(Duration::MAX);

        for item in self.iter.by_ref() {
            let delta = item.timestamp() - last_ts;

            let step_delta = (self.step_duration - delta).abs();

            if step_delta < min_delta {
                min_delta = step_delta;
                self.buffer.push_back((delta, item));
            } else {
                // if we find an element where the delta increases, we know we have the desired
                // element in the back of the buffer, so pop it, then truncate the remaining
                // elements, then finally push the current 'item' back in so we don't skip it next
                // iteration. If the buffer is empty, push the new item in, but skip to the next
                // iteration of 'self.iter' to make sure the next item isn't the better item.
                if let Some(return_item) = self.buffer.pop_back() {
                    self.buffer.clear();
                    self.buffer.push_back((delta, item));
                    return Some(return_item);
                } else {
                    self.buffer.push_back((delta, item));
                }
            }
        }

        // if we're here, that means the inner iterator is out of items, but we may still have
        // an item in the buffer that should be yielded.
        min_delta = Duration::MAX;
        let mut last_item = None;
        while let Some((delta, item)) = self.buffer.pop_front() {
            let step_delta = (self.step_duration - delta).abs();

            if step_delta < min_delta {
                min_delta = step_delta;
                last_item = Some((delta, item));
            } else {
                self.last_item_ts = Some(item.timestamp());
                self.buffer.push_front((delta, item));
                return last_item;
            }
        }

        if let Some((_, item)) = last_item.as_ref() {
            self.last_item_ts = Some(item.timestamp());
        }

        last_item
    }
}
