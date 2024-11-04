use std::fmt;
use std::hint::unreachable_unchecked;

use serde::de;

/// A [`de::Visitor`]/[`de::DeserializeSeed`] that can never be constructed
/// (by containing a [`!`] field).
pub struct NeverVisitor<T> {
    #[allow(dead_code)] // cant access '!', which is sorta the point
    never: !,
    marker: std::marker::PhantomData<T>,
}

impl<T> fmt::Debug for NeverVisitor<T> {
    fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
        // SAFETY: NeverVisitor can never be constructed, therefore this code can never run
        unsafe { unreachable_unchecked() }
    }
}

impl<'de, T> de::Visitor<'de> for NeverVisitor<T> {
    type Value = T;

    fn expecting(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
        // SAFETY: NeverVisitor can never be constructed, therefore this code can never run
        unsafe { unreachable_unchecked() }
    }
}

impl<'de, T> de::DeserializeSeed<'de> for NeverVisitor<T> {
    type Value = T;

    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // SAFETY: NeverVisitor can never be constructed, therefore this code can never run
        unsafe { unreachable_unchecked() }
    }
}
