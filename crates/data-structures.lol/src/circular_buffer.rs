//! A push-only circular buffer, with a fixed capacity.

use std::fmt;
use std::mem::{self, MaybeUninit};
use std::ops::{Index, IndexMut};

pub struct CircularBuffer<const CAP: usize, T> {
    buf: [MaybeUninit<T>; CAP],
    len: usize,
    // the next index that a push will write to
    tail: usize,
}

impl<const CAP: usize, T> Clone for CircularBuffer<CAP, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut buf = crate::uninit_array();

        for (idx, elem) in self.as_discontinuous_slice().iter().enumerate() {
            buf[idx].write(elem.clone());
        }

        Self {
            buf,
            tail: self.tail,
            len: self.len,
        }
    }
}

struct DebugRepr<'a, const CAP: usize, T> {
    inner: &'a CircularBuffer<CAP, T>,
}

struct DebugListRepr<'a, const CAP: usize, T> {
    inner: &'a CircularBuffer<CAP, T>,
}

impl<const CAP: usize, T> fmt::Debug for DebugListRepr<'_, CAP, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut list_dbg = formatter.debug_list();

        for i in 0..self.inner.len {
            list_dbg.entry(unsafe { self.inner.buf[i].assume_init_ref() });
        }

        for _ in self.inner.len..CAP {
            list_dbg.entry(&"uninitialized");
        }

        list_dbg.finish()
    }
}

impl<const CAP: usize, T> fmt::Debug for DebugRepr<'_, CAP, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("CircularBuffer")
            .field("buf", &self.inner.as_debug_list_repr())
            .field("len", &self.inner.len)
            .field("tail", &self.inner.tail)
            .finish()
    }
}

impl<const CAP: usize, T> CircularBuffer<CAP, T> {
    pub const CAPACITY: usize = CAP;

    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: crate::uninit_array(),
            len: 0,
            tail: 0,
        }
    }

    fn as_debug_list_repr(&self) -> DebugListRepr<'_, CAP, T> {
        DebugListRepr { inner: self }
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    #[allow(dead_code)]
    fn as_debug_repr(&self) -> DebugRepr<'_, CAP, T> {
        DebugRepr { inner: self }
    }

    pub fn from_array(array: [T; CAP]) -> Self {
        Self {
            buf: array.map(MaybeUninit::new),
            len: CAP,
            tail: CAP,
        }
    }

    // returns the inner slice, which may not be ordered as expected due to the tail wraping
    // around 'CAP'.
    pub fn as_discontinuous_slice(&self) -> &[T] {
        // SAFETY: 0..self.len is always initialized, since we cant pop any items.
        unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[..self.len]) }
    }

    #[inline]
    pub fn make_contiguous(&mut self) -> &mut [T] {
        // SAFETY: 0..self.len is always initialized
        let buf = unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf[..self.len]) };

        // if we arent full, or if tail == self.len, then 0..self.len is contiguous
        if self.len < CAP || self.tail == self.len {
            return buf;
        }

        let rotate_by = self.len - self.tail;

        // rotate so the tail is now at the end
        buf.rotate_right(rotate_by);
        self.tail = (self.tail + rotate_by) % CAP;
        buf
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        unsafe {
            let to_tail = MaybeUninit::slice_assume_init_ref(&self.buf[0..self.tail]);
            let after_tail = MaybeUninit::slice_assume_init_ref(&self.buf[self.tail..self.len]);

            (to_tail, after_tail)
        }
    }

    #[inline]
    const fn wrap_index(&self, offset: usize) -> usize {
        let start = if self.is_full() { self.tail } else { 0 };
        (start + offset) % self.len
    }

    /// Clears the CircularBuffer, dropping anything that it contained.
    pub fn clear(&mut self) {
        for elem in self.buf[..self.len].iter_mut() {
            let _ = unsafe { mem::replace(elem, MaybeUninit::uninit()).assume_init() };
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == CAP
    }

    pub fn iter(&self) -> Iter<'_, T> {
        let (leading, trailing) = self.as_slices();
        Iter {
            leading: leading.iter(),
            trailing: trailing.iter(),
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        let old = mem::replace(&mut self.buf[self.tail], MaybeUninit::new(item));

        if self.is_full() {
            // SAFETY: since we only push items to this collection, as long as we're 'full'
            // the replaced item is initialized and needs to be dropped.
            let _ = unsafe { old.assume_init() };
        }

        self.tail = (self.tail + 1) % CAP;
        self.len = (self.len + 1).min(CAP);
    }
}

impl<const CAP: usize, T> IntoIterator for CircularBuffer<CAP, T> {
    type Item = T;
    type IntoIter = IntoIter<CAP, T>;

    fn into_iter(self) -> Self::IntoIter {
        let head = if self.is_full() { self.tail } else { 0 };

        IntoIter { inner: self, head }
    }
}

pub struct IntoIter<const CAP: usize, T> {
    inner: CircularBuffer<CAP, T>,
    head: usize,
}

impl<const CAP: usize, T> Iterator for IntoIter<CAP, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.len == 0 {
            return None;
        }

        let next = mem::replace(&mut self.inner.buf[self.head], MaybeUninit::uninit());

        self.head = (self.head + 1) % CAP;
        self.inner.len -= 1;

        Some(unsafe { next.assume_init() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }
}

impl<const CAP: usize, T> fmt::Debug for CircularBuffer<CAP, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl<const CAP: usize, T> ExactSizeIterator for IntoIter<CAP, T> {}

impl<const CAP: usize, T> DoubleEndedIterator for IntoIter<CAP, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.inner.len == 0 {
            return None;
        }

        let next = mem::replace(&mut self.inner.buf[self.inner.tail], MaybeUninit::uninit());

        self.inner.tail = (self.inner.tail - 1) % CAP;
        self.inner.len -= 1;

        Some(unsafe { next.assume_init() })
    }
}

impl<const CAP: usize, T> Default for CircularBuffer<CAP, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize, T> Drop for CircularBuffer<CAP, T> {
    fn drop(&mut self) {
        self.clear()
    }
}

pub struct Iter<'a, T> {
    leading: std::slice::Iter<'a, T>,
    trailing: std::slice::Iter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.leading.next().or_else(|| self.trailing.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.leading.len() + self.trailing.len()
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.trailing
            .next_back()
            .or_else(|| self.leading.next_back())
    }
}

impl<const CAP: usize, T> Index<usize> for CircularBuffer<CAP, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "out of bounds index");

        if !self.is_full() {
            // SAFETY: if the index is less than the length, and we arent full, 0..len is
            // initialized
            unsafe { self.buf[index].assume_init_ref() }
        } else {
            // if full, all elements are initialized
            unsafe { self.buf[self.wrap_index(index)].assume_init_ref() }
        }
    }
}

impl<const CAP: usize, T> IndexMut<usize> for CircularBuffer<CAP, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "out of bounds index");

        if !self.is_full() {
            // SAFETY: if the index is less than the length, and we arent full, 0..len is
            // initialized
            unsafe { self.buf[index].assume_init_mut() }
        } else {
            // if full, all elements are initialized
            unsafe { self.buf[self.wrap_index(index)].assume_init_mut() }
        }
    }
}

impl<const CAP: usize, T> FromIterator<T> for CircularBuffer<CAP, T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut new = Self::new();
        new.extend(iter);
        new
    }
}

impl<const CAP: usize, T> Extend<T> for CircularBuffer<CAP, T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            self.push(item);
        }
    }
}

#[test]
fn test_circ_buf() {
    let mut buf: CircularBuffer<5, i32> = CircularBuffer::new();

    assert!(buf.is_empty());
    assert!(buf.make_contiguous().is_empty());

    buf.extend(0..3);

    assert_eq!(buf.as_slices(), ([0_i32, 1, 2].as_slice(), [].as_slice()));
    assert!(!buf.is_full() && !buf.is_empty());

    buf.extend(3..6);
    assert!(buf.is_full());

    // the original 0 should have been overwritten, so we start at 1.
    assert_eq!(buf[0], 1);

    buf.extend(6..9);

    assert_eq!(
        buf.as_slices(),
        ([5_i32, 6, 7, 8].as_slice(), [4_i32].as_slice())
    );

    assert_eq!(buf.make_contiguous(), &[4_i32, 5, 6, 7, 8]);

    let mut iter = buf.clone().into_iter();

    assert_eq!(iter.len(), 5);
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.len(), 4);

    assert!(iter.eq([5, 6, 7, 8]));
}
