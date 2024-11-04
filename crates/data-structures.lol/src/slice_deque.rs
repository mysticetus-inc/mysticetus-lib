//! [`SliceDeque`], a wrapper around any slice providing pop-only deque-like operations.

use std::borrow::Borrow;
use std::ops::{Deref, Index};

/// A wrapper around `&[T]` that supports "pop"-only operations on the front/back, like a
/// deque. These are psuedo-deque operations, since this wrapper acts more like a head/tail cursor,
/// and no elements are actually removed from the underlying slice.
///
/// Implements [`AsRef<[T]>`], [`Deref`] (where `Deref::Target = [T]`), [`Borrow<[T]>`], and
/// [`Index<Idx>`] (where `[T]: Index<Idx>`).
///
/// ```
/// # use data_structures::slice_deque::SliceDeque;
/// let string = "neat";
///
/// let mut deque = SliceDeque::new(string.as_bytes());
///
/// assert_eq!(deque.len(), 4);
/// assert_eq!(deque.pop_front(), Some(&b'n'));
/// assert_eq!(deque.pop_back(), Some(&b't'));
/// assert_eq!(deque.pop_front(), Some(&b'e'));
/// assert_eq!(deque.pop_front(), Some(&b'a'));
/// assert!(deque.is_empty());
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SliceDeque<'s, T>(&'s [T]);

impl<'s, T> Clone for SliceDeque<'s, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'s, T> Copy for SliceDeque<'s, T> {}

impl<'s, T> SliceDeque<'s, T> {
    /// Constructs a new [`SliceDeque`] from a slice.
    pub const fn new(slice: &'s [T]) -> Self {
        Self(slice)
    }

    /// Returns a reference to the first item in the slice, then removes the element from the
    /// internal slice Uses [`[]::split_first`] under the hood.
    ///
    /// ```
    /// # use data_structures::slice_deque::SliceDeque;
    /// let nums: Vec<u8> = vec![0, 1, 2];
    ///
    /// let mut deque = SliceDeque::new(nums.as_slice());
    ///
    /// assert_eq!(deque.pop_front(), Some(&0));
    /// assert_eq!(deque.pop_front(), Some(&1));
    /// assert_eq!(deque.pop_front(), Some(&2));
    /// assert_eq!(deque.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<&'s T> {
        match self.0.split_first() {
            Some((first, rem)) => {
                self.0 = rem;
                Some(first)
            }
            _ => None,
        }
    }

    /// Returns a reference to the last item in the slice, then removes the element from the
    /// internal slice. Uses [`[]::split_last`] under the hood.
    ///
    /// ```
    /// # use data_structures::slice_deque::SliceDeque;
    /// let nums: Vec<u8> = vec![2, 1];
    ///
    /// let mut deque = SliceDeque::new(nums.as_slice());
    ///
    /// assert_eq!(deque.pop_back(), Some(&1));
    /// assert_eq!(deque.pop_back(), Some(&2));
    /// assert_eq!(deque.pop_back(), None);
    /// ```
    pub fn pop_back(&mut self) -> Option<&'s T> {
        match self.0.split_last() {
            Some((last, rem)) => {
                self.0 = rem;
                Some(last)
            }
            _ => None,
        }
    }

    /// Provides a way to peek at the first element, and then conditionally determine if it
    /// should be popped or not.
    ///
    /// ```
    /// # use data_structures::slice_deque::SliceDeque;
    /// let nums: Vec<u8> = vec![2, 1];
    ///
    /// let mut deque = SliceDeque::new(nums.as_slice());
    ///
    /// let peek = deque.peek_front().unwrap();
    ///
    /// // Implements `Deref` for the peeked element.
    /// assert_eq!(&*peek, &2);
    /// assert_eq!(peek.pop(), &2);
    ///
    /// // calling `Peek::pop` removes it from the deque.
    /// assert_eq!(deque.as_ref(), &[1]);
    /// ```
    pub fn peek_front(&mut self) -> Option<PeekFront<'s, '_, T>> {
        self.0.first().map(|first| PeekFront {
            deque: self,
            peeked: first,
        })
    }

    /// Provides a way to peek at the first element, and then conditionally determine if it
    /// should be popped or not.
    ///
    /// ```
    /// # use data_structures::slice_deque::SliceDeque;
    /// let nums: Vec<u8> = vec![2, 1];
    ///
    /// let mut deque = SliceDeque::new(nums.as_slice());
    ///
    /// let peek = deque.peek_back().unwrap();
    ///
    /// // Implements `Deref` for the peeked element.
    /// assert_eq!(&*peek, &1);
    /// assert_eq!(peek.pop(), &1);
    ///
    /// // calling `Peek::pop` removes it from the deque.
    /// assert_eq!(deque.as_ref(), &[2]);
    /// ```
    pub fn peek_back(&mut self) -> Option<PeekBack<'s, '_, T>> {
        self.0.last().map(|last| PeekBack {
            deque: self,
            peeked: last,
        })
    }

    pub fn iter(&self) -> SliceDequeIter<'_, T> {
        (*self).into_iter()
    }
}

macro_rules! impl_peeks {
    ($($peek:ident => $pop_fn:ident),* $(,)?) => {
        $(
            /// A type that peeks into a [`SliceDeque`], and gives the ability for deferred
            /// popping.
            #[derive(Debug, PartialEq, Eq)]
            pub struct $peek<'a, 'd, T> {
                deque: &'d mut SliceDeque<'a, T>,
                peeked: &'a T,
            }

            impl<'a, 'd, T> $peek<'a, 'd, T> {
                #[doc = "Consumes [`"]
                #[doc = stringify!($peek)]
                #[doc = "`]"]
                /// popping the element from the [`SliceDeque`] and returning the
                /// reference.
                pub fn pop(self) -> &'a T {
                    self.deque.$pop_fn();
                    self.peeked
                }
            }

            impl<T> Deref for $peek<'_, '_, T> {
                type Target = T;

                fn deref(&self) -> &Self::Target {
                    self.peeked
                }
            }
        )*
    };
}

impl_peeks! {
    PeekFront => pop_front,
    PeekBack => pop_back,
}

pub struct SliceDequeIter<'a, T> {
    inner: SliceDeque<'a, T>,
}

impl<'a, T> Iterator for SliceDequeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

impl<'a, T> DoubleEndedIterator for SliceDequeIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop_back()
    }
}

impl<T> ExactSizeIterator for SliceDequeIter<'_, T> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, T> IntoIterator for SliceDeque<'a, T> {
    type Item = &'a T;
    type IntoIter = SliceDequeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SliceDequeIter { inner: self }
    }
}

impl<'a, T> From<&'a [T]> for SliceDeque<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self(slice)
    }
}

impl<'a, T> From<SliceDeque<'a, T>> for &'a [T] {
    fn from(deque: SliceDeque<'a, T>) -> Self {
        deque.0
    }
}

impl<T, Idx> Index<Idx> for SliceDeque<'_, T>
where
    [T]: Index<Idx>,
{
    type Output = <[T] as Index<Idx>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T> AsRef<[T]> for SliceDeque<'_, T> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<T> Borrow<[T]> for SliceDeque<'_, T> {
    fn borrow(&self) -> &[T] {
        self.0
    }
}

impl<T> Deref for SliceDeque<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/*
pub struct SliceDequeMut<'a, T> {
    slice: &'a mut [T],
    head: usize,
    tail: usize,
}

impl<'a, T> SliceDequeMut<'a, T> {

}

*/
