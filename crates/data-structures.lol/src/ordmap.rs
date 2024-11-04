use std::cmp::Ordering;
use std::fmt;

/// An ordered map, optimized for lookup performance. Internally maintains a sorted
/// [`Vec`] with key/value pairs, which lets us use binary searches for key lookups.
pub struct OrdMap<K, V> {
    pairs: Vec<(K, V)>,
}

impl<K, V> fmt::Debug for OrdMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for OrdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> OrdMap<K, V> {
    #[inline]
    pub const fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pairs: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.pairs.clear()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.pairs.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            inner: self.pairs.iter_mut(),
        }
    }
}

impl<K, V> IntoIterator for OrdMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.pairs.into_iter(),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a OrdMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut OrdMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[inline]
fn extract_key<K, V>(tup: &(K, V)) -> &K {
    &tup.0
}

impl<K, V> OrdMap<K, V>
where
    K: Ord,
{
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.pairs.binary_search_by_key(&&key, extract_key) {
            Ok(idx) => Some(std::mem::replace(&mut self.pairs[idx].1, value)),
            Err(insert_idx) => {
                self.pairs.insert(insert_idx, (key, value));

                #[cfg(debug_assertions)]
                self.ensure_sorted();

                None
            }
        }
    }

    /// Test method to ensure that keys are both unique and sorted.
    #[cfg(debug_assertions)]
    fn ensure_sorted(&self) {
        for [(leading, _), (trailing, _)] in self.pairs.array_windows::<2>() {
            match leading.cmp(trailing) {
                Ordering::Less => (),
                Ordering::Equal => panic!("found equal elements while ensuring sorted pairs"),
                Ordering::Greater => panic!("found out of order keys while ensuring sorted pairs"),
            }
        }
    }

    pub fn from_pairs(mut pairs: Vec<(K, V)>) -> Self {
        pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
        pairs.dedup_by(|(a, _), (b, _)| a == b);
        Self { pairs }
    }

    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.pairs.binary_search_by_key(&key, extract_key).is_ok()
    }

    #[inline]
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        match self.pairs.binary_search_by_key(&key, extract_key) {
            Ok(idx) => Some(self.pairs.remove(idx)),
            _ => None,
        }
    }

    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.remove_entry(key) {
            Some((_key, value)) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_pairs(&self) -> &[(K, V)] {
        self.pairs.as_slice()
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        match self.pairs.binary_search_by_key(&key, extract_key) {
            Ok(idx) => Some(&self.pairs[idx].1),
            Err(_) => None,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.pairs.binary_search_by_key(&key, extract_key) {
            Ok(idx) => Some(&mut self.pairs[idx].1),
            Err(_) => None,
        }
    }
}

impl<K, V> Extend<(K, V)> for OrdMap<K, V>
where
    K: Ord,
{
    #[inline]
    fn extend_one(&mut self, item: (K, V)) {
        self.insert(item.0, item.1);
    }

    #[inline]
    fn extend_reserve(&mut self, additional: usize) {
        self.pairs.reserve(additional);
    }

    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        /*
        let init_len = self.pairs.len();

        let mut pushed_out_of_order = false;


        for (k, v) in iter {
            // we know that all elements that existed before are ordered, so look
            let (ordered, unordered) = self.pairs.split_at_mut(init_len);

            if let Ok(idx) = ordered.binary_search_by_key(&&k, extract_key) {
                ordered[idx].1 = v;
            }
            else if let Some((_, dst)) = unordered
                .iter_mut()
                .find(|(existing_key, _)| K::cmp(existing_key, &k).is_eq())
            {
                *dst = v;
            }
            else {
                pushed_out_of_order = true;
                self.pairs.push((k, v));
            }
        }

        if pushed_out_of_order {
            self.pairs.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));
        }

        #[cfg(debug_assertions)]
        self.ensure_sorted();
        */

        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V> FromIterator<(K, V)> for OrdMap<K, V>
where
    K: Ord,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let (low, high) = iter.size_hint();

        let mut dst = OrdMap::with_capacity(high.unwrap_or(low));
        dst.extend(iter);
        dst
    }
}

#[inline]
fn invert_tuple_refs<A, B>(tup: &(A, B)) -> (&A, &B) {
    (&tup.0, &tup.1)
}

#[inline]
fn invert_tuple_mut_refs<A, B>(tup: &mut (A, B)) -> (&A, &mut B) {
    (&tup.0, &mut tup.1)
}

pub struct Iter<'a, K, V> {
    inner: std::slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(invert_tuple_refs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(invert_tuple_refs)
    }
}

pub struct IterMut<'a, K, V> {
    inner: std::slice::IterMut<'a, (K, V)>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(invert_tuple_mut_refs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(invert_tuple_mut_refs)
    }
}

pub struct IntoIter<K, V> {
    inner: std::vec::IntoIter<(K, V)>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, K, V> arbitrary::Arbitrary<'a> for OrdMap<K, V>
where
    K: Ord + arbitrary::Arbitrary<'a>,
    V: arbitrary::Arbitrary<'a>,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.arbitrary_len::<(K, V)>()?;

        let mut dst = OrdMap::with_capacity(len);
        for _ in 0..len {
            let key = K::arbitrary(u)?;
            let value = V::arbitrary(u)?;
            dst.insert(key, value);
        }

        Ok(dst)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use std::fmt;
    use std::marker::PhantomData;

    use serde::{Serialize, de};

    use super::OrdMap;

    impl<K, V> Serialize for OrdMap<K, V>
    where
        K: Serialize,
        V: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeMap;

            let mut map_ser = serializer.serialize_map(Some(self.len()))?;

            for (key, value) in self.iter() {
                map_ser.serialize_entry(key, value)?;
            }

            map_ser.end()
        }
    }

    impl<'de, K, V> de::Deserialize<'de> for OrdMap<K, V>
    where
        K: Ord + de::Deserialize<'de>,
        V: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(Visitor(PhantomData))
        }
    }

    struct Visitor<K, V>(PhantomData<(K, V)>);

    impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
    where
        K: Ord + de::Deserialize<'de>,
        V: de::Deserialize<'de>,
    {
        type Value = OrdMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of values")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut dst = seq
                .size_hint()
                .map(OrdMap::with_capacity)
                .unwrap_or_default();

            while let Some((k, v)) = seq.next_element::<(K, V)>()? {
                dst.insert(k, v);
            }

            Ok(dst)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut dst = map
                .size_hint()
                .map(OrdMap::with_capacity)
                .unwrap_or_default();

            while let Some((k, v)) = map.next_entry::<K, V>()? {
                dst.insert(k, v);
            }

            Ok(dst)
        }
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "arbitrary")]
    #[test]
    fn test_ordmap() -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();

        let mut bytes: Vec<u8> = vec![0; 1000];
        rng.fill(bytes.as_mut_slice());

        let mut arb = arbitrary::Unstructured::new(bytes.as_slice());

        let ordmap: OrdMap<char, usize> = arbitrary::Arbitrary::arbitrary(&mut arb)?;

        println!("{ordmap:#?}");

        Ok(())
    }
}
