use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// A shared type. Can either be a <code>&'static T</code>, or an [`Arc<T>`]. All trait impls
/// defer to the underlying <code>T</code> (rather, a reference to the underlying <code>T</code>).
pub enum Shared<T: ?Sized + 'static> {
    /// A static Shared type.
    Static(&'static T),
    /// An [`Arc`] type.
    Arc(Arc<T>),
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(s) => Self::Static(*s),
            Self::Arc(a) => Self::Arc(Arc::clone(a)),
        }
    }
}

impl<T: PartialEq + ?Sized> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T: PartialEq + ?Sized> PartialEq<T> for Shared<T> {
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<T: Eq + ?Sized> Eq for Shared<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for Shared<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T: PartialOrd + ?Sized> PartialOrd<T> for Shared<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T: Ord + ?Sized> Ord for Shared<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized> Shared<T> {
    pub fn is_static(&self) -> bool {
        matches!(*self, Self::Static(_))
    }

    pub fn as_ptr(&self) -> *const T {
        match self {
            Self::Static(s) => *s,
            Self::Arc(a) => Arc::as_ptr(a),
        }
    }

    pub fn is_same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => std::ptr::eq(*a, *b),
            (Self::Arc(a), Self::Arc(b)) => std::ptr::eq(Arc::as_ptr(a), Arc::as_ptr(b)),
            _ => false,
        }
    }

    pub fn is_arc(&self) -> bool {
        matches!(*self, Self::Arc(_))
    }

    pub fn as_arc(&self) -> Option<&Arc<T>> {
        match self {
            Self::Arc(arc) => Some(arc),
            Self::Static(_) => None,
        }
    }

    #[inline]
    pub const fn as_static(&self) -> Option<&'static T> {
        match *self {
            Self::Static(stat) => Some(stat),
            _ => None,
        }
    }

    #[inline]
    pub fn into_arc(self) -> Option<Arc<T>> {
        match self {
            Self::Arc(arc) => Some(arc),
            _ => None,
        }
    }

    #[inline]
    pub fn into_owned(self) -> T
    where
        T: Sized + ToOwned<Owned = T>,
    {
        match self {
            Self::Static(s) => s.to_owned(),
            Self::Arc(arc) => match Arc::try_unwrap(arc) {
                Ok(unwrapped) => unwrapped,
                Err(arc) => T::to_owned(&arc),
            },
        }
    }
}

impl<T: ?Sized> From<&'static T> for Shared<T> {
    #[inline]
    fn from(stat: &'static T) -> Self {
        Self::Static(stat)
    }
}

impl From<String> for Shared<str> {
    fn from(value: String) -> Self {
        Self::Arc(Arc::from(value))
    }
}

impl From<Cow<'static, str>> for Shared<str> {
    fn from(value: Cow<'static, str>) -> Self {
        match value {
            Cow::Borrowed(b) => Self::Static(b),
            Cow::Owned(s) => Self::Arc(Arc::from(s)),
        }
    }
}

impl<T: ?Sized> From<Arc<T>> for Shared<T> {
    #[inline]
    fn from(arc: Arc<T>) -> Self {
        Self::Arc(arc)
    }
}

impl<T> From<T> for Shared<T> {
    #[inline]
    fn from(owned: T) -> Self {
        Self::Arc(Arc::new(owned))
    }
}

impl<T: ?Sized> AsRef<T> for Shared<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        match self {
            Self::Static(s) => s,
            Self::Arc(s) => s,
        }
    }
}

impl<T: ?Sized> Borrow<T> for Shared<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> fmt::Debug for Shared<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self.as_ref(), f)
    }
}

impl<T: ?Sized> fmt::Display for Shared<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self.as_ref(), f)
    }
}
