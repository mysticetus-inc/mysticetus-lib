use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

/// A [`Cow`]-like enum, where it can either be an owned <code>T</code>, or a reference to a
/// <code>T</code>.
///
/// Unlike [`Cow`], both variants are of type <code>T</code>, removing the [`Clone`]/[`ToOwned`]
/// bound that [`Cow`] requires.
///
/// This wrapping type implements things like [`PartialEq`]/[`PartialOrd`]/[`Eq`]/[`Ord`]/[`Hash`]
/// /[`Display`]/[`Deserialize`]/[`Serialize`] in such a way that it defers to those
/// implementations on <code>T</code> itself, meaning:
///
/// ```
/// # use data_structures::maybe_owned_mut::MaybeOwnedMut;
/// let mut value: u8 = 42;
/// let owned = MaybeOwnedMut::Owned(value);
/// let refer = MaybeOwnedMut::MutRef(&mut value);
///
/// assert_eq!(refer, owned);
/// assert_eq!(refer, 42);
/// assert_eq!(owned, 42);
/// ```
///
/// The [`Clone`] impl defers to the [`Clone`] impl for <code>T</code>, making it owned if it was
/// previously a mutable reference.
///
/// [`Display`]: fmt::Display
/// [`Cow`]: std::borrow::Cow
#[derive(Debug)]
pub enum MaybeOwnedMut<'a, T> {
    /// An owned <code>T</code>.
    Owned(T),
    /// A mutable reference to a <code>T</code>.
    MutRef(&'a mut T),
}

impl<'a, T> From<&'a mut T> for MaybeOwnedMut<'a, T> {
    fn from(refer: &'a mut T) -> Self {
        Self::MutRef(refer)
    }
}

impl<T> From<T> for MaybeOwnedMut<'static, T> {
    fn from(owned: T) -> Self {
        Self::Owned(owned)
    }
}

impl<T> Default for MaybeOwnedMut<'static, T>
where
    T: Default,
{
    fn default() -> Self {
        MaybeOwnedMut::Owned(T::default())
    }
}

impl<T> fmt::Display for MaybeOwnedMut<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), formatter)
    }
}

impl<T> Hash for MaybeOwnedMut<'_, T>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_ref().hash(state);
    }
}

impl<T> Borrow<T> for MaybeOwnedMut<'_, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> MaybeOwnedMut<'_, T> {
    /// Returns whether or not this [`MaybeOwnedMut`] is [`MaybeOwnedMut::Owned`].
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Returns whether or not this [`MaybeOwnedMut`] is [`MaybeOwnedMut::MutRef`].
    pub fn is_mut_ref(&self) -> bool {
        matches!(self, Self::MutRef(_))
    }

    /// Returns [`&mut T`].
    #[inline]
    pub const fn as_mut(&mut self) -> &mut T {
        match self {
            Self::Owned(owned) => owned,
            Self::MutRef(mut_ref) => mut_ref,
        }
    }
}

impl<'a, T> MaybeOwnedMut<'a, T> {
    pub fn get_ref(&mut self) -> MaybeOwnedMut<'_, T> {
        MaybeOwnedMut::MutRef(self.as_mut())
    }
}

impl<T> MaybeOwnedMut<'_, T>
where
    T: Deref,
{
    pub fn to_deref(&self) -> &T::Target {
        self.deref()
    }
}

impl<'a, T> MaybeOwnedMut<'a, Option<T>> {
    pub fn as_ref(&self) -> Option<&T> {
        match AsRef::<Option<T>>::as_ref(self) {
            Some(refer) => Some(refer),
            None => None,
        }
    }

    pub fn as_deref(&self) -> Option<&<T as Deref>::Target>
    where
        T: Deref,
    {
        match self {
            MaybeOwnedMut::MutRef(Some(refer)) => Some(refer.deref()),
            MaybeOwnedMut::Owned(Some(owned)) => Some(owned.deref()),
            _ => None,
        }
    }

    pub fn transpose(self) -> Option<MaybeOwnedMut<'a, T>> {
        match self {
            Self::MutRef(Some(refer)) => Some(MaybeOwnedMut::MutRef(refer)),
            Self::Owned(Some(owned)) => Some(MaybeOwnedMut::Owned(owned)),
            Self::MutRef(None) | Self::Owned(None) => None,
        }
    }
}

impl<'a, T> MaybeOwnedMut<'a, T>
where
    T: Clone,
{
    /// Consumes self and returns <code>T</code>, cloning if self is [`MaybeOwnedMut::MutRef`].
    pub fn into_owned(self) -> T {
        match self {
            Self::MutRef(refer) => refer.clone(),
            Self::Owned(owned) => owned,
        }
    }

    /// If 'self' is [`MaybeOwnedMut::MutRef`], the value is cloned and stored back in 'self' as a
    /// [`MaybeOwnedMut::Owned`]. If 'self' was already [`MaybeOwnedMut::Owned`], this is a no-op.
    pub fn make_owned(&mut self) {
        if let Self::MutRef(refer) = self {
            *self = Self::Owned(refer.clone());
        }
    }

    /// Identical functionality to [`Cow::to_mut`]. Clones the inner value if needed, and
    /// returns a mutable reference to it.
    ///
    /// [`Cow::to_mut`]: std::borrow::Cow::to_mut
    pub fn to_mut(&mut self) -> &mut T {
        self.make_owned();

        match self {
            Self::Owned(owned) => owned,
            // SAFETY: calling self.make_owned() above enforces that this is an [`Owned`] variant.
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn as_owned(self) -> MaybeOwnedMut<'static, T> {
        MaybeOwnedMut::Owned(self.into_owned())
    }
}

impl<T> Deref for MaybeOwnedMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            Self::Owned(owned) => owned,
            Self::MutRef(refer) => refer,
        }
    }
}

impl<T> DerefMut for MaybeOwnedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> AsRef<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for MaybeOwnedMut<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> Clone for MaybeOwnedMut<'_, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Owned(owned) => Self::Owned(owned.clone()),
            Self::MutRef(refer) => Self::Owned(T::clone(refer)),
        }
    }
}

impl<T> PartialEq for MaybeOwnedMut<'_, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T> PartialEq<T> for MaybeOwnedMut<'_, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<T> Eq for MaybeOwnedMut<'_, T> where T: Eq {}

impl<T> PartialOrd for MaybeOwnedMut<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> PartialOrd<T> for MaybeOwnedMut<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T> Ord for MaybeOwnedMut<'_, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Serialize for MaybeOwnedMut<'_, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize(self.as_ref(), serializer)
    }
}

impl<'de, T> Deserialize<'de> for MaybeOwnedMut<'static, T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(MaybeOwnedMut::Owned)
    }
}
