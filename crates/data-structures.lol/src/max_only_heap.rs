use std::cmp::{Ordering, Reverse};

/// A type alias to [`MaxOnlyHeap<T, Reverse<F>>`], to only track the minimum.
pub type MinOnlyHeap<T, F> = MaxOnlyHeap<T, Reverse<F>>;

/// A trait that describes a way to extract a key from an item. This key is what [`MaxOnlyHeap`]
/// uses to determine what to hold onto.
pub trait KeyExtractor<T> {
    /// The type of key. Must be [`Ord`].
    type Key: Ord;

    /// Performs the key extraction from an instance of <code>T</code>.
    fn extract_key(&self, item: &T) -> Self::Key;
}

/// A marker struct that represents a tuple (up to a 12-tuple).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tuple;

macro_rules! impl_tuple_extractor {
    ($key:ident, $next_t:ident, $($t:ident,)* $(,)?) => {
        impl<$key, $next_t, $($t,)*> KeyExtractor<($key, $next_t, $($t,)*)> for Tuple
        where
            $key: Ord + Clone
        {
            type Key = $key;
            fn extract_key(&self, item: &($key, $next_t, $($t,)*)) -> Self::Key {
                item.0.clone()
            }
        }

        impl_tuple_extractor!($key, $($t,)*);
    };
    ($key:ident $(,)?) => {
        // stop condition
    }
}

impl_tuple_extractor!(K, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,);

impl<F, T> KeyExtractor<T> for Reverse<F>
where
    F: KeyExtractor<T>,
{
    type Key = Reverse<F::Key>;

    fn extract_key(&self, item: &T) -> Reverse<F::Key> {
        Reverse(self.0.extract_key(item))
    }
}

impl<F, T, K> KeyExtractor<T> for F
where
    F: Fn(&T) -> K,
    K: Ord,
{
    type Key = K;

    fn extract_key(&self, item: &T) -> K {
        (self)(item)
    }
}

/// A [`KeyExtractor`] representing that [`Self`] should be used itself as the ordering key.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelfKey;

impl<T> KeyExtractor<T> for SelfKey
where
    T: Clone + Ord,
{
    type Key = T;

    fn extract_key(&self, extract_key: &T) -> T {
        extract_key.clone()
    }
}

/// A trait representing a type that can produce it's own [`Key`] internally.
///
/// [`Key`]: Keyed::Key
pub trait Keyed {
    /// The key type.
    type Key: Ord;

    /// The key getter.
    fn key(&self) -> Self::Key;
}

impl<T> KeyExtractor<T> for ()
where
    T: Keyed,
{
    type Key = T::Key;

    fn extract_key(&self, item: &T) -> Self::Key {
        item.key()
    }
}

/// A max-heap that contains only elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaxOnlyHeap<T, F>
where
    F: KeyExtractor<T>,
{
    key: Option<F::Key>,
    items: Vec<T>,
    key_extractor: F,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushOp<T> {
    NotInserted(T),
    Inserted,
}

impl<T> MaxOnlyHeap<T, SelfKey>
where
    T: Ord + Copy,
{
    pub const fn new_self_key() -> Self {
        Self::new(SelfKey)
    }
}

impl<T> MaxOnlyHeap<T, ()>
where
    T: Keyed,
{
    pub const fn new_keyed() -> Self {
        Self::new(())
    }
}

impl<K, T> MaxOnlyHeap<(K, T), Tuple>
where
    K: Ord + Copy,
{
    pub const fn new_tuple() -> Self {
        Self::new(Tuple)
    }
}

pub type OldKeyItemsPair<K, T> = Option<(K, Vec<T>)>;

impl<T, F> MaxOnlyHeap<T, F>
where
    F: KeyExtractor<T>,
{
    /// Creates a new [`MaxOnlyHeap`] with the specified [`KeyExtractor`]
    pub const fn new(key_extractor: F) -> Self {
        Self {
            key: None,
            items: Vec::new(),
            key_extractor,
        }
    }

    /// Creates a new [`MaxOnlyHeap`] with a pre-allocated capacity.
    pub fn with_capacity(capacity: usize, key_extractor: F) -> Self {
        Self {
            key: None,
            items: Vec::with_capacity(capacity),
            key_extractor,
        }
    }

    /// Clears the currently tracked key + items.
    pub fn clear(&mut self) {
        self.key = None;
        self.items.clear();
    }

    /// If not empty, destructures into the tracked key + items.
    pub fn into_inner(self) -> Option<(F::Key, Vec<T>)> {
        match self.key {
            Some(key) => Some((key, self.items)),
            None => None,
        }
    }

    /// If not empty, takes the currently tracked key and items.
    pub fn take(&mut self) -> Option<(F::Key, Vec<T>)> {
        match self.key.take() {
            Some(key) => Some((key, std::mem::take(&mut self.items))),
            _ => None,
        }
    }

    /// The number of items currently in the tracked collection.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether or not the tracked collection is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the currently tracked key, if not empty.
    pub fn key(&self) -> Option<&F::Key> {
        self.key.as_ref()
    }

    /// Returns the capacity of the inner collection.
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// If not empty, returns an reference to the currently tracked key + items.
    pub fn key_items(&self) -> Option<(&F::Key, &[T])> {
        self.key.as_ref().map(|key| (key, self.items.as_slice()))
    }

    /// If not empty, returns a slice of the currently tracked items.
    pub fn items(&self) -> Option<&[T]> {
        if self.key.is_some() {
            Some(&self.items)
        } else {
            None
        }
    }

    /// Attempts to pushes a new item into the inner collection. Returns a descriptor of what
    /// operation was performed, as well as the previous key/values if the new item replaced the
    /// old items.
    pub fn push_recieve(&mut self, item: T) -> (PushOp<T>, OldKeyItemsPair<F::Key, T>) {
        let key = self.key_extractor.extract_key(&item);

        match self.key.as_mut().map(|curr| (Ord::cmp(&*curr, &key), curr)) {
            None => {
                self.key = Some(key);
                self.items.push(item);
                (PushOp::Inserted, None)
            }
            Some((Ordering::Equal, _)) => {
                self.items.push(item);
                (PushOp::Inserted, None)
            }
            Some((Ordering::Greater, _)) => (PushOp::NotInserted(item), None),
            Some((Ordering::Less, curr_key)) => {
                let old_key = std::mem::replace(curr_key, key);
                let old_items = std::mem::replace(&mut self.items, vec![item]);

                (PushOp::Inserted, Some((old_key, old_items)))
            }
        }
    }

    /// Attempts to pushes a new item into the inner collection. Returns a descriptor of what
    /// operation was performed.
    pub fn push(&mut self, item: T) -> PushOp<T> {
        let key = self.key_extractor.extract_key(&item);

        match self.key.as_ref().map(|curr| curr.cmp(&key)) {
            None => {
                self.key = Some(key);
                self.items.push(item);
                PushOp::Inserted
            }
            Some(Ordering::Equal) => {
                self.items.push(item);
                PushOp::Inserted
            }
            Some(Ordering::Greater) => PushOp::NotInserted(item),
            Some(Ordering::Less) => {
                self.key = Some(key);
                self.items.clear();
                self.items.push(item);
                PushOp::Inserted
            }
        }
    }
}

impl<T, F> FromIterator<T> for MaxOnlyHeap<T, F>
where
    F: KeyExtractor<T> + Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut heap = MaxOnlyHeap::new(F::default());
        heap.extend(iter);
        heap
    }
}

impl<T, F> Extend<T> for MaxOnlyHeap<T, F>
where
    F: KeyExtractor<T>,
{
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
fn test_max_only_heap() {
    let mut heap = MaxOnlyHeap::new(SelfKey);

    heap.push(1);
    heap.push(1);
    assert_eq!(heap.len(), 2);

    assert_eq!(heap.push(0), PushOp::NotInserted(0));

    assert_eq!(heap.push(2), PushOp::Inserted);
    assert_eq!(heap.len(), 1);
}
