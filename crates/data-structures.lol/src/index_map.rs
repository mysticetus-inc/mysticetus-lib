use std::borrow::Borrow;
use std::fmt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "deepsize", derive(deepsize::DeepSizeOf))]
pub struct IndexMap<K: Ord, V> {
    entries: Box<[(K, V)]>,
}

impl<K, V> fmt::Debug for IndexMap<K, V>
where
    K: Ord + fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Ord, V> IndexMap<K, V> {
    #[inline]
    pub fn from_pairs(mut pairs: Vec<(K, V)>) -> Self {
        let mut has_dupes = false;
        pairs.sort_unstable_by(|(a, _), (b, _)| {
            let ord = a.cmp(b);
            has_dupes |= ord.is_eq();
            ord
        });

        if has_dupes {
            pairs.dedup_by(|(a, _), (b, _)| a == b);
        }

        Self {
            entries: pairs.into_boxed_slice(),
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub const fn as_pairs(&self) -> &[(K, V)] {
        &self.entries
    }

    #[inline]
    pub fn index_of<U>(&self, target: &U) -> Option<usize>
    where
        K: Borrow<U>,
        U: Ord + ?Sized,
    {
        self.entries
            .binary_search_by(|(key, _)| key.borrow().cmp(target))
            .ok()
    }

    #[inline]
    pub fn get<U>(&self, target: &U) -> Option<&V>
    where
        K: Borrow<U>,
        U: Ord + ?Sized,
    {
        self.index_of(target).map(|index| &self.entries[index].1)
    }

    #[inline]
    pub fn get_mut<U>(&mut self, target: &U) -> Option<&mut V>
    where
        K: Borrow<U>,
        U: Ord + ?Sized,
    {
        self.index_of(target)
            .map(|index| &mut self.entries[index].1)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            inner: self.entries.iter_mut(),
        }
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a IndexMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a mut IndexMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Iter<'a, K, V> {
    inner: std::slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len();

        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(k, v)| (k, v))
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct IterMut<'a, K, V> {
    inner: std::slice::IterMut<'a, (K, V)>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (&*k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len();

        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(k, v)| (&*k, v))
    }
}
