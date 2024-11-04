use std::fmt;

/// A reference only set based on [`Vec`] that preserves insertion order.
pub struct RefSet<'a, T: ?Sized> {
    insert_order: Vec<&'a T>,
    cmp_order: Vec<Inserted<'a, T>>,
}

impl<'a, T: ?Sized> Default for RefSet<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for RefSet<'_, T>
where
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl<'a, T: ?Sized> AsRef<[&'a T]> for RefSet<'a, T> {
    fn as_ref(&self) -> &[&'a T] {
        self.as_slice()
    }
}

struct Inserted<'a, T: ?Sized> {
    refer: &'a T,
    idx: usize,
}

impl<T: ?Sized> RefSet<'_, T> {
    pub const fn new() -> Self {
        Self {
            insert_order: Vec::new(),
            cmp_order: Vec::new(),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.insert_order.iter(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            insert_order: Vec::with_capacity(capacity),
            cmp_order: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.insert_order.clear();
        self.cmp_order.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.insert_order.reserve(additional);
        self.cmp_order.reserve(additional);
    }

    pub fn len(&self) -> usize {
        self.insert_order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.insert_order.is_empty()
    }
}

impl<'a, T: ?Sized> RefSet<'a, T> {
    pub fn as_slice(&self) -> &[&'a T] {
        self.insert_order.as_slice()
    }
    /// Gets items from the set in insertion order.
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: std::slice::SliceIndex<[&'a T]>,
    {
        self.insert_order.get(index)
    }
}

impl<'a, T: ?Sized> RefSet<'a, T>
where
    T: Ord,
{
    #[inline]
    pub fn insert(&mut self, item: &'a T) -> bool {
        match self
            .cmp_order
            .binary_search_by(|set_item| T::cmp(set_item.refer, item))
        {
            Ok(_) => false,
            Err(cmp_idx) => {
                let idx = self.insert_order.len();
                self.insert_order.push(item);
                self.cmp_order
                    .insert(cmp_idx, Inserted { refer: item, idx });
                true
            }
        }
    }

    pub fn index_of(&self, item: &T) -> Option<usize> {
        match self
            .cmp_order
            .binary_search_by(|set_item| T::cmp(set_item.refer, item))
        {
            Ok(idx) => Some(idx),
            _ => None,
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        self.cmp_order
            .binary_search_by(|set_item| T::cmp(set_item.refer, item))
            .is_ok()
    }

    pub fn remove(&mut self, item: &T) -> bool {
        match self
            .cmp_order
            .binary_search_by(|set_item| T::cmp(set_item.refer, item))
        {
            Ok(idx) => {
                let removed = self.cmp_order.remove(idx);
                for item in self.cmp_order.iter_mut() {
                    if item.idx > removed.idx {
                        item.idx -= 1;
                    }
                }
                self.insert_order.remove(removed.idx);
                true
            }
            Err(_) => false,
        }
    }
}

impl<'a, T> Extend<&'a T> for RefSet<'a, T>
where
    T: ?Sized + Ord,
{
    fn extend_one(&mut self, item: &'a T) {
        self.insert(item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        let iter = iter.into_iter();
        let (low, high) = iter.size_hint();
        self.reserve(high.unwrap_or(low) / 2);

        for refer in iter {
            self.insert(refer);
        }
    }
}

impl<'a, T> FromIterator<&'a T> for RefSet<'a, T>
where
    T: ?Sized + Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
    {
        let mut dst = RefSet::new();
        dst.extend(iter);
        dst
    }
}

pub struct Iter<'a, T: ?Sized> {
    inner: std::slice::Iter<'a, &'a T>,
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len();
        (len, Some(len))
    }
}

impl<'a, T, I> std::ops::Index<I> for RefSet<'a, T>
where
    T: ?Sized + Ord,
    I: std::slice::SliceIndex<[&'a T]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index).expect("out of bounds access")
    }
}

impl<T: ?Sized> ExactSizeIterator for Iter<'_, T> {}

impl<T: ?Sized> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_set() {
        let mut set = RefSet::from_iter(["a", "b", "c"]);

        assert!(set.contains("c"));
        assert!(!set.contains("v"));

        assert!(set.contains("b"));
        assert!(set.remove("b"));
        assert!(!set.contains("b"));

        assert_eq!(&set[..], &["a", "c"]);

        assert_eq!(&set[1..], &["c"]);

        assert!(!set.is_empty());
        assert_eq!(set.len(), 2);

        assert_eq!(set.iter().collect::<Vec<_>>(), vec!["a", "c"]);

        set.extend(["d", "e", "f"]);
        assert_eq!(set.len(), 5);

        assert_eq!(&set[..], &["a", "c", "d", "e", "f"]);
        assert_eq!(&set[3..], &["e", "f"]);

        assert!(set.remove("e"));
        assert!(!set.contains("e"));

        assert_eq!(set.len(), 4);
        assert_eq!(&set[..], &["a", "c", "d", "f"]);
        assert_eq!(&set[3..], &["f"]);
    }
}
