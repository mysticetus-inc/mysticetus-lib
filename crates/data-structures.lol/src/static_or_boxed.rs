use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

pub enum StaticOrBoxed<T: ?Sized + 'static> {
    Static(&'static T),
    Boxed(Box<T>),
}

impl<T: ?Sized + 'static + ToOwned> Clone for StaticOrBoxed<T>
where
    Box<T>: From<T::Owned>,
{
    fn clone(&self) -> Self {
        match self {
            Self::Static(s) => StaticOrBoxed::Static(s),
            Self::Boxed(b) => StaticOrBoxed::Boxed(Box::from(T::to_owned(b))),
        }
    }
}

impl<T: ?Sized + 'static + Hash> Hash for StaticOrBoxed<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_ref().hash(state);
    }
}

impl<T: ?Sized + 'static> From<&'static T> for StaticOrBoxed<T> {
    #[inline]
    fn from(value: &'static T) -> Self {
        Self::Static(value)
    }
}

impl<T: ?Sized + 'static> From<Box<T>> for StaticOrBoxed<T> {
    fn from(value: Box<T>) -> Self {
        Self::Boxed(value)
    }
}

impl<T: ?Sized + 'static> AsRef<T> for StaticOrBoxed<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized + 'static> Borrow<T> for StaticOrBoxed<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized + 'static> Deref for StaticOrBoxed<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(s) => s,
            Self::Boxed(b) => b,
        }
    }
}

impl<T: fmt::Debug + ?Sized + 'static> fmt::Debug for StaticOrBoxed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: fmt::Display + ?Sized + 'static> fmt::Display for StaticOrBoxed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: ?Sized + PartialEq + 'static, U: AsRef<T>> PartialEq<U> for StaticOrBoxed<T> {
    fn eq(&self, other: &U) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T: ?Sized + Eq + 'static> Eq for StaticOrBoxed<T> {}

impl<T: ?Sized + PartialOrd + 'static, U: AsRef<T>> PartialOrd<U> for StaticOrBoxed<T> {
    fn partial_cmp(&self, other: &U) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T: ?Sized + Ord + 'static> Ord for StaticOrBoxed<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}
