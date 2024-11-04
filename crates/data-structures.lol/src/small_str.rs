use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;

/// A small string optimization, based on [`smallvec`]. Can hold up to `CAP` bytes
/// on the stack, spilling over to the heap if needed.
#[derive(Clone, Hash)]
#[repr(transparent)]
pub struct SmallStr<const CAP: usize> {
    buf: SmallVec<[u8; CAP]>,
}

#[cfg(feature = "deepsize")]
impl<const CAP: usize> deepsize::DeepSizeOf for SmallStr<CAP> {
    fn deep_size_of_children(&self, _context: &mut deepsize::Context) -> usize {
        self.buf.capacity()
    }
}

impl<const CAP: usize> Default for SmallStr<CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> SmallStr<CAP> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: SmallVec::new_const(),
        }
    }

    #[inline]
    /// Explicit conversion to [`&str`]. Used internally by the [`AsRef`], [`Borrow`] and
    /// [`Deref`] impls.
    pub fn as_str(&self) -> &str {
        // SAFETY: 'buf' is only allowed to contain valid UTF-8 bytes. The methods
        // that do provide mutable access to the bytes themselves are all unsafe,
        // so those contracts must be upheld by the user.
        unsafe { std::str::from_utf8_unchecked(&self.buf[..]) }
    }

    /// Returns a mutable slice to the underlying bytes. Identical to
    /// [`str::as_bytes_mut`], including the safety contract.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    #[inline]
    /// Returns true if `Self` is not on the heap, and is inline.
    pub fn is_inline(&self) -> bool {
        !self.buf.spilled()
    }

    /// Returns a mutable slice to the underlying bytes.
    /// # Safety
    /// Identical to [`str::as_bytes_mut`], including the safety contract.
    #[inline]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    #[inline]
    pub fn push(&mut self, c: char) {
        let mut buf = [0; 4];
        let encoded = c.encode_utf8(&mut buf);
        self.push_str(encoded);
    }

    /// Converts [`self`] into an owned [`String`]. If the current value is on the stack, this
    /// allocates a new [`String`], but if already on the heap, this does no cloning.
    #[inline]
    pub fn into_string(self) -> String {
        // SAFETY: 'buf' must have valid UTF-8, the only way it can't is if the safety invariants
        // for [`SmallStr::as_bytes_mut`] were violated.
        unsafe { String::from_utf8_unchecked(self.buf.into_vec()) }
    }

    /// Takes ownership from a [`String`] and converts it to a [`SmallStr`], without
    /// copying or cloning.
    #[inline]
    pub fn from_string(s: String) -> Self {
        Self {
            buf: SmallVec::from_vec(s.into_bytes()),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

impl<const CAP: usize> std::str::FromStr for SmallStr<CAP> {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl<const CAP: usize> From<&str> for SmallStr<CAP> {
    #[inline]
    fn from(s: &str) -> Self {
        Self {
            buf: SmallVec::from_slice(s.as_bytes()),
        }
    }
}

impl<const CAP: usize> From<String> for SmallStr<CAP> {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl<const CAP: usize> From<std::borrow::Cow<'_, str>> for SmallStr<CAP> {
    #[inline]
    fn from(s: std::borrow::Cow<'_, str>) -> Self {
        match s {
            std::borrow::Cow::Owned(owned) => Self::from_string(owned),
            std::borrow::Cow::Borrowed(b) => Self::from(b),
        }
    }
}

impl<const CAP: usize> From<crate::str::BoxStr> for SmallStr<CAP> {
    #[inline]
    fn from(s: crate::str::BoxStr) -> Self {
        s.into_string().into()
    }
}

impl<const CAP: usize> fmt::Debug for SmallStr<CAP> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Debug>::fmt(self.as_str(), formatter)
    }
}

impl<const CAP: usize> fmt::Display for SmallStr<CAP> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt(self.as_str(), formatter)
    }
}

impl<const CAP: usize> AsRef<str> for SmallStr<CAP> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAP: usize> Borrow<str> for SmallStr<CAP> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const CAP: usize> Deref for SmallStr<CAP> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAP: usize, T> PartialEq<T> for SmallStr<CAP>
where
    T: AsRef<str> + ?Sized,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_str().eq(other.as_ref())
    }
}

impl<const CAP: usize> Eq for SmallStr<CAP> {}

impl<T, const CAP: usize> PartialOrd<T> for SmallStr<CAP>
where
    T: AsRef<str> + ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        Some(self.as_str().cmp(other.as_ref()))
    }
}

impl<const CAP: usize> Ord for SmallStr<CAP> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

#[cfg(feature = "serde")]
impl<const CAP: usize> Serialize for SmallStr<CAP> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de, const CAP: usize> Deserialize<'de> for SmallStr<CAP> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(Visitor)
    }
}

#[cfg(feature = "serde")]
struct Visitor<const CAP: usize>;

#[cfg(feature = "serde")]
impl<'de, const CAP: usize> de::Visitor<'de> for Visitor<CAP> {
    type Value = SmallStr<CAP>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SmallStr::from(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        std::str::from_utf8(v)
            .map(SmallStr::from)
            .map_err(|_| de::Error::invalid_type(de::Unexpected::Bytes(v), &self))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_borrowed_str(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        String::from_utf8(v)
            .map(SmallStr::from_string)
            .map_err(|e| de::Error::invalid_type(de::Unexpected::Bytes(&e.into_bytes()), &self))
    }
}

#[cfg(feature = "small-str-write")]
impl<const CAP: usize> fmt::Write for SmallStr<CAP> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}
