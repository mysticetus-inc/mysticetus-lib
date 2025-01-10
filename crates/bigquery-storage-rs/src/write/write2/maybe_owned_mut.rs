use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};

pub(super) enum MaybeOwnedMut<'a, T> {
    Owned(T),
    RefMut(&'a mut T),
}

impl<T> MaybeOwnedMut<'_, T> {
    #[inline]
    pub fn into_static(self) -> MaybeOwnedMut<'static, T>
    where
        T: Clone,
    {
        match self {
            Self::Owned(owned) => MaybeOwnedMut::Owned(owned),
            Self::RefMut(refmut) => MaybeOwnedMut::Owned(refmut.clone()),
        }
    }
}

impl<T: Default> Default for MaybeOwnedMut<'_, T> {
    #[inline]
    fn default() -> Self {
        Self::Owned(T::default())
    }
}

impl<T> From<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a mut T> for MaybeOwnedMut<'a, T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        Self::RefMut(value)
    }
}

impl<T> Deref for MaybeOwnedMut<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match self {
            Self::Owned(owned) => owned,
            Self::RefMut(mut_ref) => &**mut_ref,
        }
    }
}

impl<T> DerefMut for MaybeOwnedMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match self {
            Self::Owned(owned) => owned,
            Self::RefMut(mut_ref) => *mut_ref,
        }
    }
}

impl<T> AsRef<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<T> AsMut<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut *self
    }
}

impl<T> Borrow<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<T> BorrowMut<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut *self
    }
}
