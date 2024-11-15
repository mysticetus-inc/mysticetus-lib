//! A [`Serializer`] (not yet complete) and [`Deserializer`] wrapper, that gives paths to where
//! an error during serialization/deserialization occurs.
#![feature(let_chains, const_trait_impl)]
#![warn(missing_docs)]

mod de;
mod error;
mod path;
mod ser;

#[cfg(feature = "json")]
pub mod json;

pub use de::Deserializer;
pub use error::Error;
pub use path::{Path, Segment};
pub use ser::Serializer;

/// Extension trait for types that implement [`serde::Deserialize`].
pub trait DeserializeExt<'de>
where
    Self: serde::Deserialize<'de>,
{
    /// [`Deserialize::deserialze`], but internally wraps the passed in deserializer to provide
    /// path-aware errors. This changes the return type from `Result<Self, D::Error>` to
    /// `Result<Self, Error<D::Error>>` to account for the wrapped deserialization.
    fn deserialize_path_aware<D>(deserializer: D) -> Result<Self, Error<D::Error>>
    where
        D: serde::Deserializer<'de>,
    {
        Self::deserialize(deserializer.make_path_aware())
    }
}

impl<'de, T> DeserializeExt<'de> for T where T: serde::Deserialize<'de> {}

/// Extension trait for types that implement [`serde::Deserialize`].
pub trait DeserializerExt<'de>
where
    Self: serde::Deserializer<'de>,
{
    /// Takes a [`Deserializer`], and wraps it to provide path aware errors.
    fn make_path_aware(self) -> Deserializer<Self> {
        Deserializer::new(self)
    }
}

impl<'de, T> DeserializerExt<'de> for T where T: serde::Deserializer<'de> {}

/// Extension trait for types that implement [`serde::Serialize`].
pub trait SerializeExt
where
    Self: serde::Serialize,
{
    /// [`Serialize::serialize`], but internally wraps the passed in serializer to provide
    /// path-aware errors. This changes the return type from `Result<S::Ok, S::Error>` to
    /// `Result<S::Ok, Error<S::Error>>` to account for the wrapped deserialization.
    fn serialize_path_aware<S>(&self, serializer: S) -> Result<S::Ok, Error<S::Error>>
    where
        S: serde::Serializer,
    {
        self.serialize(serializer.make_path_aware())
    }
}

impl<T> SerializeExt for T where T: serde::Serialize {}

/// Extension trait for types that implement [`serde::Serializer`].
pub trait SerializerExt
where
    Self: serde::Serializer,
{
    /// Takes a [`serde::Serializer`], and wraps it to provide path aware errors.
    fn make_path_aware(self) -> Serializer<'static, 'static, Self> {
        Serializer::new(self)
    }
}

impl<T> SerializerExt for T where T: serde::Serializer {}
