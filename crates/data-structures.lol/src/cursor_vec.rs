//! Wrapper for [`Vec<T>`] or <code>&mut [`Vec<T>`]</code> which can generate a [`LinkedList`]-like
//! [`CursorMut`], without any of the downsides of using a [`LinkedList`]
//!
//! [`LinkedList`]: [`std::collections::LinkedList`]
//! [`CursorMut`]: [`std::collections::linked_list::CursorMut`]
use std::ops::{Deref, DerefMut};

/// Extension trait for [`Vec`]/[`&mut Vec`] that provides the [`CursorVec`] cursor functions,
/// without actually needing to wrap the inner [`Vec`] in a [`CursorVec`].
pub trait CursorVecExt: AsVec {
    #[inline]
    fn cursor(&mut self) -> Cursor<Self::Element> {
        self.cursor_at(0)
    }

    #[inline]
    fn cursor_back(&mut self) -> Cursor<Self::Element> {
        self.cursor_at(self.as_ref().len().saturating_sub(1))
    }

    #[inline]
    fn cursor_at(&mut self, start_idx: usize) -> Cursor<Self::Element> {
        Cursor {
            inner: self.as_mut(),
            current_index: start_idx,
        }
    }
}

impl<T> CursorVecExt for T where T: AsVec {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CursorVec<V: AsVec>(pub V);

impl<V> CursorVec<V>
where
    V: AsVec,
{
    #[inline]
    pub fn cursor(&mut self) -> Cursor<V::Element> {
        self.cursor_at(0)
    }

    #[inline]
    pub fn cursor_back(&mut self) -> Cursor<V::Element> {
        self.cursor_at(self.0.as_ref().len().saturating_sub(1))
    }

    #[inline]
    pub fn cursor_at(&mut self, start_idx: usize) -> Cursor<V::Element> {
        Cursor {
            inner: self.0.as_mut(),
            current_index: start_idx,
        }
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, V::Element> {
        self.0.as_ref().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, V::Element> {
        self.0.as_mut().iter_mut()
    }
}

impl<V> IntoIterator for CursorVec<V>
where
    V: AsVec + IntoIterator,
{
    type Item = V::Item;
    type IntoIter = V::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Supertrait for [`AsRef<Vec<T>>`] + [`AsMut<Vec<T>>`].
///
/// The [`AsVec::Element`] associated type lets us reference the element type without adding a
/// 2nd generic parameter for the element.
pub trait AsVec: AsRef<Vec<Self::Element>> + AsMut<Vec<Self::Element>> {
    type Element;
}

impl<T> AsVec for Vec<T> {
    type Element = T;
}

impl<V> AsVec for &mut V
where
    V: AsVec,
{
    type Element = <V as AsVec>::Element;
}

impl<V: AsVec> AsRef<[V::Element]> for CursorVec<V> {
    #[inline]
    fn as_ref(&self) -> &[V::Element] {
        self.0.as_ref().as_slice()
    }
}

impl<V: AsVec> AsMut<[V::Element]> for CursorVec<V> {
    #[inline]
    fn as_mut(&mut self) -> &mut [V::Element] {
        self.0.as_mut().as_mut_slice()
    }
}

impl<V: AsVec> Deref for CursorVec<V> {
    type Target = Vec<V::Element>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<V: AsVec> DerefMut for CursorVec<V> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cursor<'a, T> {
    inner: &'a mut Vec<T>,
    current_index: usize,
}

impl<T> Cursor<'_, T> {
    #[inline]
    pub fn current(&self) -> Option<&T> {
        self.inner.get(self.current_index)
    }

    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut(self.current_index)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.current_index
    }

    #[inline]
    pub fn inner_ref(&self) -> &Vec<T> {
        &*self.inner
    }

    /// equivalent to [`Vec::remove`], at the index this cursor points at. Unlike the [`Vec`]
    /// implementation, this checks that the current index is within bounds, causing this to
    /// return [`Option<T>`]. If ordering does not need to be preserved, [`swap_remove`]
    /// is more performant. After removing, the index will be pointing at the next element. If the
    /// removed element was previously the final element, this will then be pointing off the end
    /// of the vec, requiring a call to [`move_back`]/[`move_offset`] to get back on track.
    ///
    /// [`swap_remove`]: Cursor::swap_remove    
    /// [`move_back`]: Cursor::move_back
    /// [`move_offset`]: Cursor::move_offset    
    #[inline]
    pub fn remove(&mut self) -> Option<T> {
        if self.current_index >= self.inner.len() {
            return None;
        }

        let elem = self.inner.remove(self.current_index);

        // if we removed the final element in the vec, move backwards 1 so we're still pointing at
        // the vec (assuming it's not empty, will saturate at 0 if so).
        if self.current_index == self.inner.len() {
            self.current_index = self.current_index.saturating_sub(1);
        }

        Some(elem)
    }

    /// equivalent to [`Vec::swap_remove`], at the index this cursor points at. Unlike the [`Vec`]
    /// implementation, this checks that the current index is within bounds, causing this to
    /// return [`Option<T>`]. After removing, the current index will be pointing at the element
    /// that took its place (the one from the end).
    #[inline]
    pub fn swap_remove(&mut self) -> Option<T> {
        if self.current_index >= self.inner.len() {
            return None;
        }

        let elem = self.inner.swap_remove(self.current_index);

        // if we removed the final element in the vec, move backwards 1 so we're still pointing at
        // the vec (assuming it's not empty, will saturate at 0 if so).
        if self.current_index == self.inner.len() {
            self.current_index = self.current_index.saturating_sub(1);
        }

        Some(elem)
    }

    /// Manually set the index that the cursor is pointing at. No checks are done to make sure
    /// it's within the bounds of the [`Vec`].
    #[inline]
    pub fn move_to_index(&mut self, index: usize) {
        self.current_index = index;
    }

    /// Move the index, relative to the current position. Saturates at 0, as well as at
    /// 'length - 1'. This aims to always have the current index pointing at a valid element
    /// (assuming the inner [`Vec`] isn't empty).
    #[inline]
    pub fn move_offset(&mut self, offset: isize) {
        self.current_index = self.current_index.saturating_add_signed(offset);
    }

    /// Similar to [`move_next`], but wraps around to the front if incrementing the index leaves
    /// us pointing at an element that doesn't exist, i.e '>= [`Vec::len`]'. If called while
    /// pointing to an element at an index '0..len - 1', this behaves identically to
    /// [`move_next`]. This can come in handy in conjunction with [`remove`]/[`swap_remove`] being
    /// called on the final element, as it'll put the cursor back at the beginning.
    ///
    /// [`move_next`]: Cursor::move_next
    /// [`remove`]: Cursor::remove
    /// [`swap_remove`]: Cursor::swap_remove
    #[inline]
    pub fn move_next_wrapping(&mut self) {
        if self.current_index + 1 < self.inner.len() {
            self.move_next();
        } else {
            self.current_index = 0;
        }
    }

    /// Similar idea to [`move_next_wrapping`], but backwards. If pointing at the first element,
    /// calling this will wrap us back to the final element in the array. Otherwise is identical
    /// to [`move_back`].
    ///
    /// [`move_next_wrapping`]: Cursor::move_next_wrapping
    /// [`move_back`]: Cursor::move_back    
    #[inline]
    pub fn move_back_wrapping(&mut self) {
        if self.current_index > 0 {
            self.move_back();
        } else {
            self.current_index = self.inner.len() - 1;
        }
    }

    /// Shortcut for [`.move_offset(1)`].
    #[inline]
    pub fn move_next(&mut self) {
        self.move_offset(1)
    }

    /// Shortcut for [`.move_offset(-1)`].
    #[inline]
    pub fn move_back(&mut self) {
        self.move_offset(-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_vec() {
        let mut cursor_vec = CursorVec(vec![0, 1, 2, 3, 4]);

        let mut cursor = cursor_vec.cursor();

        assert_eq!(Some(0), cursor.remove());

        cursor.move_offset(2);

        assert_eq!(Some(&3), cursor.current());

        cursor.move_next_wrapping();
        assert_eq!(Some(&4), cursor.current());

        cursor.move_next_wrapping();
        assert_eq!(Some(&1), cursor.current());

        cursor.move_back_wrapping();
        assert_eq!(Some(&4), cursor.current());

        cursor.move_next();
        assert_eq!(None, cursor.current());

        cursor.move_next_wrapping();
        assert_eq!(Some(&1), cursor.current());
    }
}
