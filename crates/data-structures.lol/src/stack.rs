//! An array based stack.

use std::cmp::Ordering;
use std::fmt;
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut, Index, IndexMut};

pub struct Stack<const CAP: usize, T> {
    array: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<const CAP: usize, T> fmt::Debug for Stack<CAP, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<const CAP: usize, T> Clone for Stack<CAP, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new = Self::new();

        let dst = unsafe { MaybeUninit::slice_assume_init_mut(&mut new.array[..self.len]) };

        dst.clone_from_slice(self.as_slice());

        new.len = self.len;

        new
    }
}

impl<const CAP: usize, T> Stack<CAP, T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            array: MaybeUninit::uninit_array(),
            len: 0,
        }
    }

    /// # Panics
    /// [`Stack::remaining_capacity`] must be greater or equal to the length of 'slice'.
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Copy,
    {
        MaybeUninit::copy_from_slice(&mut self.array[self.len..][..slice.len()], slice);
    }

    /// Safer version of [`Stack::extend_from_slice`]. Truncates 'slice' if longer than
    /// [`Stack::remaining_capacity`], which prevents panics. If all of 'slice' is written,
    /// [`Ok`] is returned, otherwise [`Err`] is returned with the number of elements copied.
    pub fn extend_from_slice_truncate(&mut self, slice: &[T]) -> Result<(), usize>
    where
        T: Copy,
    {
        if self.remaining_capacity() >= slice.len() {
            self.extend_from_slice(slice);
            Ok(())
        } else {
            let subslice = &slice[..self.remaining_capacity()];
            self.extend_from_slice(subslice);
            Err(subslice.len())
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn remaining_capacity(&self) -> usize {
        CAP.saturating_sub(self.len)
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == CAP
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    #[inline]
    pub const fn push(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        self.array[self.len].write(item);
        self.len += 1;

        Ok(())
    }

    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.len -= 1;

        // SAFETY: if we're not empty, the value at len - 1 (before the above line)
        // is initialized.
        unsafe {
            Some(mem::replace(&mut self.array[self.len], MaybeUninit::uninit()).assume_init())
        }
    }

    #[inline]
    pub const fn remove(&mut self, mut index: usize) -> Option<T> {
        if self.len() >= index {
            return None;
        }

        let out = unsafe {
            std::mem::replace(&mut self.array[index], MaybeUninit::uninit()).assume_init()
        };

        self.len -= 1;

        while self.len() > index {
            unsafe {
                std::ptr::swap(
                    self.array[index].as_mut_ptr(),
                    self.array[index + 1].as_mut_ptr(),
                );
            }

            index += 1;
        }

        Some(out)
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if self.len() >= index {
            return None;
        }

        // SAFETY: we did the bound check above
        unsafe {
            std::ptr::swap(
                self.array[index].as_mut_ptr(),
                self.array[self.len - 1].as_mut_ptr(),
            );
        }

        self.pop()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: up to len - 1 (saturating at 0) is initialized
        unsafe {
            let subslice = &self.array[..self.len];
            MaybeUninit::slice_assume_init_ref(subslice)
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.len {
            unsafe {
                std::mem::replace(&mut self.array[i], MaybeUninit::uninit()).assume_init();
            }
        }

        self.len = 0;
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: up to len is initialized
        unsafe {
            let subslice = &mut self.array[..self.len];
            MaybeUninit::slice_assume_init_mut(subslice)
        }
    }
}

impl<const CAP: usize, T> Deref for Stack<CAP, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const CAP: usize, T> DerefMut for Stack<CAP, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<const CAP: usize, T> AsRef<[T]> for Stack<CAP, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<const CAP: usize, T> AsMut<[T]> for Stack<CAP, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<const CAP: usize, T> Default for Stack<CAP, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize, T> Index<usize> for Stack<CAP, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);
        // SAFETY: assertion prevents UB
        unsafe { self.array[index].assume_init_ref() }
    }
}

impl<const CAP: usize, T> IndexMut<usize> for Stack<CAP, T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);
        // SAFETY: assertion prevents UB
        unsafe { self.array[index].assume_init_mut() }
    }
}

impl<const CAP: usize, T> FromIterator<T> for Stack<CAP, T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut new = Self::new();
        new.extend(iter);
        new
    }
}

impl<const CAP: usize, T> Extend<T> for Stack<CAP, T> {
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            if self.push(item).is_err() {
                break;
            }
        }
    }
}

impl<const CAP: usize, T, S> PartialEq<S> for Stack<CAP, T>
where
    S: AsRef<[T]>,
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &S) -> bool {
        <[T] as PartialEq>::eq(self.as_slice(), other.as_ref())
    }
}

impl<const CAP: usize, T, S> PartialOrd<S> for Stack<CAP, T>
where
    S: AsRef<[T]>,
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &S) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}

impl<const CAP: usize, T> Eq for Stack<CAP, T> where T: Eq {}

impl<const CAP: usize, T> Ord for Stack<CAP, T>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

pub struct IntoIter<const CAP: usize, T> {
    stack: Stack<CAP, T>,
}

impl<const CAP: usize, T> IntoIterator for Stack<CAP, T> {
    type Item = T;
    type IntoIter = IntoIter<CAP, T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { stack: self }
    }
}

impl<const CAP: usize, T> Iterator for IntoIter<CAP, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.swap_remove(0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.stack.len();
        (len, Some(len))
    }
}

impl<const CAP: usize, T> ExactSizeIterator for IntoIter<CAP, T> {}

impl<const CAP: usize, T> DoubleEndedIterator for IntoIter<CAP, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use std::fmt;

    use serde::{Deserialize, Serialize};

    use super::Stack;

    impl<const CAP: usize, T> Serialize for Stack<CAP, T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeSeq;

            let mut seq = serializer.serialize_seq(Some(self.len()))?;

            for item in self.iter() {
                seq.serialize_element(item)?;
            }

            seq.end()
        }
    }

    impl<'de, const CAP: usize, T> Deserialize<'de> for Stack<CAP, T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(StackVisitor(std::marker::PhantomData))
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct StackVisitor<const CAP: usize, T>(std::marker::PhantomData<([(); CAP], T)>);

    impl<'de, const CAP: usize, T> serde::de::Visitor<'de> for StackVisitor<CAP, T>
    where
        T: Deserialize<'de>,
    {
        type Value = Stack<CAP, T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a sequence of {CAP} elements or less")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut new = Stack::new();

            while let Some(next) = seq.next_element()? {
                if new.push(next).is_err() {
                    break;
                }
            }

            Ok(new)
        }
    }
}

impl<const CAP: usize, T> Drop for Stack<CAP, T> {
    fn drop(&mut self) {
        for i in 0..self.len {
            // SAFETY: all elements up to 'len' are initialized
            unsafe { self.array[i].assume_init_drop() };
        }

        self.len = 0;
    }
}
