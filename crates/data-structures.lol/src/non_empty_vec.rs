//! A [`Vec`]-like alternative that can never be empty

use std::fmt;
use std::num::NonZeroUsize;

#[derive(Clone, Hash)]
pub struct NonEmptyVec<T> {
    first: T,
    remainder: Vec<T>,
}

impl<T: fmt::Debug> fmt::Debug for NonEmptyVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entry(&self.first)
            .entries(self.remainder.iter())
            .finish()
    }
}

impl<T: Eq> Eq for NonEmptyVec<T> {}

impl<U, T> PartialEq<NonEmptyVec<U>> for NonEmptyVec<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &NonEmptyVec<U>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().eq(other.iter())
    }
}

impl<T, U> PartialEq<[U]> for NonEmptyVec<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().eq(other.iter())
    }
}

impl<T, S> PartialEq<S> for NonEmptyVec<T>
where
    T: PartialEq,
    S: AsRef<[T]>,
{
    fn eq(&self, other: &S) -> bool {
        if self.len() != other.as_ref().len() {
            return false;
        }

        self.iter().eq(other.as_ref().iter())
    }
}

impl<T> NonEmptyVec<T> {
    pub const fn new(first: T) -> Self {
        Self {
            first,
            remainder: Vec::new(),
        }
    }

    pub const fn from_parts(first: T, remainder: Vec<T>) -> Self {
        Self { first, remainder }
    }

    pub fn into_parts(self) -> (T, Vec<T>) {
        (self.first, self.remainder)
    }

    pub fn append(&mut self, v: &mut Vec<T>) {
        self.remainder.append(v);
    }

    pub fn len(&self) -> usize {
        self.remainder.len() + 1
    }

    pub const fn is_empty(&self) -> bool {
        // that's the whole point...
        false
    }

    pub fn len_non_zero(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.len()).expect(
            "len is always 1 or more, and vec capacity limits will prevent len from overflowing",
        )
    }

    pub fn into_vec(mut self) -> Vec<T> {
        self.remainder.insert(0, self.first);
        self.remainder
    }

    pub fn with_capacity(first: T, capacity: usize) -> Self {
        Self {
            first,
            remainder: Vec::with_capacity(capacity),
        }
    }

    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_by(Ord::cmp)
    }

    pub fn sort_by<F>(&mut self, mut comparator: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        // easy return.
        if self.remainder.is_empty() {
            return;
        }

        // first, sort the items in the vec
        self.remainder.sort_by(&mut comparator);

        // then, find where the first elem needs to go
        match self
            .remainder
            .binary_search_by(|item| comparator(&self.first, item))
        {
            // if it's equal to the first element, or we need to 'insert' it before, we're done
            Ok(0) | Err(0) => (),
            Ok(idx) | Err(idx) => {
                // if the insertion index is the length, we can just rotate the vec by 1 element,
                // then swap the 2 elements around
                if idx == self.remainder.len() {
                    self.remainder.rotate_left(1);
                    std::mem::swap(&mut self.first, &mut self.remainder[idx - 1]);
                } else {
                    // swap the non-vec item into the leading spot
                    std::mem::swap(&mut self.first, &mut self.remainder[0]);
                    // then re-sort the remainder vec. There's 100% a better solution here with
                    // fancy indexing or element partitioning, but since every
                    // other element is already sorted this should be fairly
                    // fast.
                    //
                    // We can use the insertion index returned by the binary search to at least
                    // reduce the 'n' in the O(n^2) sort.
                    self.remainder[..=idx].sort_by(&mut comparator);
                }
            }
        }
    }

    pub fn first(&self) -> &T {
        &self.first
    }

    pub fn first_mut(&mut self) -> &mut T {
        &mut self.first
    }

    pub fn last(&self) -> &T {
        self.remainder.last().unwrap_or(&self.first)
    }

    pub fn last_mut(&mut self) -> &mut T {
        self.remainder.last_mut().unwrap_or(&mut self.first)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.remainder.pop()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            first: Some(&self.first),
            rem: self.remainder.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            first: Some(&mut self.first),
            rem: self.remainder.iter_mut(),
        }
    }
}

// -------------------- Iter --------------------- //

pub struct Iter<'a, T> {
    first: Option<&'a T>,
    rem: std::slice::Iter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.rem.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rem.len() + self.first.is_some() as usize;

        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rem.next_back().or_else(|| self.first.take())
    }
}

impl<'a, T> IntoIterator for &'a NonEmptyVec<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// -------------------- IterMut ----------------------- //
pub struct IterMut<'a, T> {
    first: Option<&'a mut T>,
    rem: std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.rem.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rem.len() + self.first.is_some() as usize;

        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rem.next_back().or_else(|| self.first.take())
    }
}

impl<'a, T> IntoIterator for &'a mut NonEmptyVec<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// -------------------- IntoIter ----------------------- //
pub struct IntoIter<T> {
    first: Option<T>,
    rem: std::vec::IntoIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.rem.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rem.len() + self.first.is_some() as usize;

        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rem.next_back().or_else(|| self.first.take())
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            first: Some(self.first),
            rem: self.remainder.into_iter(),
        }
    }
}
