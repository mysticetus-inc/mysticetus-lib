use std::fmt;

use protos::firestore::value::ValueType;

use crate::error::SerError;
use crate::ser;

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Reference(str);

impl serde::Serialize for Reference {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize with a specific name, so the serializer can attempt to coerce this
        // to a firestore reference type instead of just a string
        serializer.serialize_newtype_struct(Self::MARKER, self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Box<Reference> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Reference::new_string)
    }
}

impl<'de> serde::Deserialize<'de> for &'de Reference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&'de str as serde::Deserialize<'de>>::deserialize(deserializer).map(Reference::new)
    }
}

impl PartialOrd for Reference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Reference {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        crate::util::cmp_paths(self.as_str(), other.as_str())
    }
}

impl AsRef<str> for Reference {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Reference {
    pub(crate) const MARKER: &str = "__firestore_reference__";

    pub fn new(s: &str) -> &Self {
        // SAFETY: We're repr(transparent)
        unsafe { std::mem::transmute::<&str, &Self>(s) }
    }

    pub(crate) fn try_serialize<W: ser::WriteKind>(
        value: &(impl serde::Serialize + ?Sized),
    ) -> Result<ValueType, SerError> {
        use ValueType::{NullValue, ReferenceValue, StringValue};

        match ser::serialize_value::<W>(value)? {
            StringValue(s) | ReferenceValue(s) => Ok(ReferenceValue(s)),
            null @ NullValue(_) => Ok(null),
            non_null => {
                // panic in debug, otherwise just pass it on
                if cfg!(debug_assertions) {
                    panic!("expected newtype struct to be a string/reference, got: {non_null:?}");
                }

                Ok(non_null)
            }
        }
    }

    pub fn id(&self) -> &str {
        let s = self.as_str();
        s.rsplit_once('/').map(|(_, id)| id).unwrap_or(s)
    }

    pub fn new_string(s: String) -> Box<Self> {
        Self::new_owned(s.into_boxed_str())
    }

    pub fn new_owned(s: Box<str>) -> Box<Self> {
        // SAFETY: We're repr(transparent)
        unsafe { std::mem::transmute::<Box<str>, Box<Self>>(s) }
    }

    pub fn into_boxed_str(self: Box<Self>) -> Box<str> {
        // SAFETY: We're repr(transparent)
        unsafe { std::mem::transmute(self) }
    }

    pub fn into_string(self: Box<Self>) -> String {
        Self::into_boxed_str(self).into_string()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Clone for Box<Reference> {
    fn clone(&self) -> Self {
        Reference::to_owned(self)
    }
}

impl ToOwned for Reference {
    type Owned = Box<Reference>;

    fn to_owned(&self) -> Self::Owned {
        Self::new_owned(Box::from(&self.0))
    }
}
