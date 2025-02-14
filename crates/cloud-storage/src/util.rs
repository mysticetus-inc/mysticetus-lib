use std::fmt;
use std::ops::{Deref, DerefMut};

pub(crate) enum OwnedOrMut<'a, T> {
    Owned(T),
    Mut(&'a mut T),
}

impl<T: fmt::Debug> fmt::Debug for OwnedOrMut<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&*self, f)
    }
}

impl<T> OwnedOrMut<'_, T> {
    #[inline]
    pub fn into_owned(self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Owned(owned) => owned,
            Self::Mut(refer) => refer.clone(),
        }
    }

    #[inline]
    pub fn into_static(self) -> OwnedOrMut<'static, T>
    where
        T: Clone,
    {
        OwnedOrMut::Owned(self.into_owned())
    }
}

impl<T: Default> Default for OwnedOrMut<'_, T> {
    #[inline]
    fn default() -> Self {
        Self::Owned(T::default())
    }
}

impl<T> AsRef<T> for OwnedOrMut<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> AsMut<T> for OwnedOrMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T> Deref for OwnedOrMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Mut(refer) => refer,
            Self::Owned(owned) => owned,
        }
    }
}

impl<T> DerefMut for OwnedOrMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Mut(refer) => refer,
            Self::Owned(owned) => owned,
        }
    }
}

impl<T> From<T> for OwnedOrMut<'_, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a mut T> for OwnedOrMut<'a, T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        Self::Mut(value)
    }
}
