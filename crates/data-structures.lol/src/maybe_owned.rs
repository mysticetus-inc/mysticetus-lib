use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

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
/// # use data_structures::maybe_owned::MaybeOwned;
/// let value: u8 = 42;
/// let refer = MaybeOwned::Ref(&value);
/// let owned = MaybeOwned::Owned(value);
///
/// assert_eq!(refer, owned);
/// assert_eq!(refer, value);
/// assert_eq!(owned, value);
/// ```
///
/// The [`Clone`] impl defers to the [`Clone`] impl for `T` if owned, otherwise it just copies
/// the reference.
///
/// [`Display`]: [`fmt::Display`]
#[derive(Debug)]
pub enum MaybeOwned<'a, T> {
    /// An owned <code>T</code>.
    Owned(T),
    /// A reference to a <code>T</code>.
    Ref(&'a T),
}

impl<'a, T> From<Cow<'a, T>> for MaybeOwned<'a, T>
where
    T: Clone,
{
    fn from(cow: Cow<'a, T>) -> Self {
        match cow {
            Cow::Borrowed(b) => MaybeOwned::Ref(b),
            Cow::Owned(o) => MaybeOwned::Owned(o),
        }
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(refer: &'a T) -> Self {
        Self::Ref(refer)
    }
}

impl<T> From<T> for MaybeOwned<'static, T> {
    fn from(owned: T) -> Self {
        Self::Owned(owned)
    }
}

impl<T> Default for MaybeOwned<'static, T>
where
    T: Default,
{
    fn default() -> Self {
        MaybeOwned::Owned(T::default())
    }
}

impl<T> fmt::Display for MaybeOwned<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), formatter)
    }
}

impl<T> Hash for MaybeOwned<'_, T>
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

impl<T> Borrow<T> for MaybeOwned<'_, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> MaybeOwned<'_, T> {
    /// Returns whether or not this [`MaybeOwned`] is [`MaybeOwned::Owned`].
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Returns whether or not this [`MaybeOwned`] is [`MaybeOwned::Ref`].
    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    /// Returns [`&mut T`] if 'self' is the [`MaybeOwned::Owned`] variant, [`None`] otherwise.
    /// To get a garunteed mutable reference via a [`Clone`] fallback, see [`to_mut`].
    ///
    /// [`to_mut`]: [`Self::to_mut`]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Owned(owned) => Some(owned),
            _ => None,
        }
    }
}

impl<'a, T> MaybeOwned<'a, T> {
    pub fn get_ref(&self) -> MaybeOwned<'_, T> {
        MaybeOwned::Ref(self.as_ref())
    }
}

impl<T> MaybeOwned<'_, T>
where
    T: Deref,
{
    pub fn to_deref(&self) -> &T::Target {
        self.deref()
    }
}

impl<'a, T> MaybeOwned<'a, Option<T>> {
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
            MaybeOwned::Ref(Some(refer)) => Some(refer.deref()),
            MaybeOwned::Owned(Some(owned)) => Some(owned.deref()),
            _ => None,
        }
    }

    pub fn transpose(self) -> Option<MaybeOwned<'a, T>> {
        match self {
            Self::Ref(Some(refer)) => Some(MaybeOwned::Ref(refer)),
            Self::Owned(Some(owned)) => Some(MaybeOwned::Owned(owned)),
            Self::Ref(None) | Self::Owned(None) => None,
        }
    }
}

impl<'a, T> MaybeOwned<'a, T>
where
    T: Clone,
{
    /// Consumes self and returns <code>T</code>, cloning if self is [`MaybeOwned::Ref`].
    pub fn into_owned(self) -> T {
        match self {
            Self::Ref(refer) => refer.clone(),
            Self::Owned(owned) => owned,
        }
    }

    /// If 'self' is [`MaybeOwned::Ref`], the value is cloned and stored back in 'self' as a
    /// [`MaybeOwned::Owned`]. If 'self' was already [`MaybeOwned::Owned`], this is a no-op.
    pub fn make_owned(&mut self) {
        if let Self::Ref(refer) = self {
            *self = Self::Owned(refer.clone());
        }
    }

    /// Identical functionality to [`Cow::to_mut`]. Clones the inner value if needed, and
    /// returns a mutable reference to it.
    pub fn to_mut(&mut self) -> &mut T {
        self.make_owned();

        match self {
            Self::Owned(owned) => owned,
            // SAFETY: calling self.make_owned() above enforces that this is an [`Owned`] variant.
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    /// Converts into a [`Cow`], with [`MaybeOwned::Owned`] mapping to [`Cow::Owned`], and
    /// [`MaybeOwned::Ref`] to [`Cow::Borrowed`].
    pub fn into_cow(self) -> Cow<'a, T> {
        match self {
            Self::Owned(owned) => Cow::Owned(owned),
            Self::Ref(refer) => Cow::Borrowed(refer),
        }
    }

    pub fn as_owned(self) -> MaybeOwned<'static, T> {
        MaybeOwned::Owned(self.into_owned())
    }
}

impl<T> Deref for MaybeOwned<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            Self::Owned(owned) => owned,
            Self::Ref(refer) => refer,
        }
    }
}

impl<T> AsRef<T> for MaybeOwned<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Clone for MaybeOwned<'_, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Owned(owned) => Self::Owned(owned.clone()),
            Self::Ref(refer) => Self::Ref(refer),
        }
    }
}

impl<T> Copy for MaybeOwned<'_, T> where T: Copy {}

impl<T> PartialEq for MaybeOwned<'_, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T> PartialEq<T> for MaybeOwned<'_, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<T> Eq for MaybeOwned<'_, T> where T: Eq {}

impl<T> PartialOrd for MaybeOwned<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> PartialOrd<T> for MaybeOwned<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T> Ord for MaybeOwned<'_, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Serialize for MaybeOwned<'_, T>
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

impl<'de, T> Deserialize<'de> for MaybeOwned<'static, T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(MaybeOwned::Owned)
    }
}
