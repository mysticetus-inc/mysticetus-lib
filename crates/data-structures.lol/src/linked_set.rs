//! An ordered set based on a linked list.
use std::cmp::Ordering;
use std::collections::{LinkedList, linked_list};

/// A linked list with ordered, unique nodes.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkedSet<T> {
    inner: LinkedList<T>,
}

impl<T> LinkedSet<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self {
            inner: LinkedList::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn insert(&mut self, item: T) -> Option<T> {
        let mut cursor = self.inner.cursor_front_mut();

        loop {
            let current = match cursor.current() {
                Some(current) => current,
                None => {
                    cursor.insert_after(item);
                    return None;
                }
            };

            match Ord::cmp(current, &item) {
                Ordering::Equal => return Some(std::mem::replace(current, item)),
                Ordering::Less => {
                    cursor.insert_before(item);
                    return None;
                }
                Ordering::Greater => {
                    cursor.move_next();
                }
            }
        }
    }

    pub fn iter(&self) -> linked_list::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn cursor(&mut self) -> Cursor<'_, T> {
        Cursor {
            inner: self.inner.cursor_front_mut(),
        }
    }
}

impl<T> IntoIterator for LinkedSet<T> {
    type Item = T;
    type IntoIter = linked_list::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

pub struct Cursor<'a, T> {
    inner: linked_list::CursorMut<'a, T>,
}

impl<T> Cursor<'_, T> {
    pub fn move_next(&mut self) {
        self.inner.move_next();
    }

    pub fn current(&self) -> Option<&T> {
        self.inner.as_cursor().current()
    }

    pub fn remove(&mut self) -> Option<T> {
        self.inner.remove_current()
    }
}
