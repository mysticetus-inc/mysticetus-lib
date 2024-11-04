use serde::{de, ser};

/// Trait abstracting over anything that has an [`is_empty`] method.
pub trait IsEmpty: private::IsEmptySealed {}

impl<T> IsEmpty for T where T: private::IsEmptySealed {}

mod private {
    use std::borrow::Cow;

    pub trait IsEmptySealed {
        fn is_empty(&self) -> bool;
    }

    impl<T> IsEmptySealed for [T] {
        #[inline]
        fn is_empty(&self) -> bool {
            <[T]>::is_empty(self)
        }
    }

    impl IsEmptySealed for str {
        #[inline]
        fn is_empty(&self) -> bool {
            str::is_empty(self)
        }
    }

    impl<K, V> IsEmptySealed for std::collections::HashMap<K, V> {
        #[inline]
        fn is_empty(&self) -> bool {
            std::collections::HashMap::is_empty(self)
        }
    }

    impl<T> IsEmptySealed for std::collections::HashSet<T> {
        #[inline]
        fn is_empty(&self) -> bool {
            std::collections::HashSet::is_empty(self)
        }
    }

    impl<K, V> IsEmptySealed for std::collections::BTreeMap<K, V> {
        #[inline]
        fn is_empty(&self) -> bool {
            std::collections::BTreeMap::is_empty(self)
        }
    }

    impl<T> IsEmptySealed for std::collections::BTreeSet<T> {
        #[inline]
        fn is_empty(&self) -> bool {
            std::collections::BTreeSet::is_empty(self)
        }
    }

    impl<T> IsEmptySealed for std::collections::LinkedList<T> {
        #[inline]
        fn is_empty(&self) -> bool {
            std::collections::LinkedList::is_empty(self)
        }
    }

    impl<T> IsEmptySealed for Vec<T> {
        #[inline]
        fn is_empty(&self) -> bool {
            <[T]>::is_empty(self.as_slice())
        }
    }

    impl<T> IsEmptySealed for Cow<'_, T>
    where
        T: ToOwned + IsEmptySealed + ?Sized,
        <T as ToOwned>::Owned: IsEmptySealed,
    {
        #[inline]
        fn is_empty(&self) -> bool {
            match self {
                Cow::Borrowed(b) => b.is_empty(),
                Cow::Owned(o) => o.is_empty(),
            }
        }
    }

    impl<T> IsEmptySealed for Box<T>
    where
        T: IsEmptySealed + ?Sized,
    {
        #[inline]
        fn is_empty(&self) -> bool {
            T::is_empty(&**self)
        }
    }

    impl<T> IsEmptySealed for Option<T>
    where
        T: IsEmptySealed,
    {
        #[inline]
        fn is_empty(&self) -> bool {
            match self.as_ref() {
                Some(inner) => inner.is_empty(),
                None => true,
            }
        }
    }

    impl<T> IsEmptySealed for &T
    where
        T: IsEmptySealed + ?Sized,
    {
        #[inline]
        fn is_empty(&self) -> bool {
            T::is_empty(*self)
        }
    }
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: de::Deserialize<'de> + IsEmpty,
    D: de::Deserializer<'de>,
{
    let s = T::deserialize(deserializer)?;

    if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
}

pub fn serialize<T, S>(opt: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ser::Serialize,
    S: ser::Serializer,
{
    match opt.as_ref() {
        Some(inner) => inner.serialize(serializer),
        None => serializer.serialize_none(),
    }
}
