//! GeoJson [`Deserializer`] that keeps track of errors.
//!
//! [`Deserializer`]: [`serde::de::Deserializer`]
use std::fmt;

use serde::de;

mod builder;
mod delegate;
mod deserializer;
mod json;
#[cfg(test)]
mod tests;

pub use builder::DeserializerBuilder;
use deserializer::DeserializerImpl;

use crate::Error;
use crate::path::{ErrorPath, Track};

/// Noop key modifier function.
#[inline(always)]
pub(crate) fn noop(_: &mut String) {}

/// A wrapper around another [`serde::Deserializer`], providing path-aware errors.
pub struct Deserializer<D> {
    inner_de: Option<D>,
    error_path: ErrorPath,
    key_modifier: fn(&mut String),
}

impl<'de, D> From<DeserializerBuilder<D>> for Deserializer<D>
where
    D: de::Deserializer<'de>,
{
    #[inline]
    fn from(builder: DeserializerBuilder<D>) -> Self {
        let init = if builder.root.is_empty() {
            None
        } else {
            Some(builder.root)
        };

        Self {
            inner_de: Some(builder.inner_de),
            error_path: ErrorPath::new(init),
            key_modifier: builder.key_modifier,
        }
    }
}

impl<D> fmt::Debug for Deserializer<D> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Deserializer")
            .field("error_path", &self.error_path)
            .field("key_modifier", &std::any::type_name::<fn(&mut String)>())
            .finish_non_exhaustive()
    }
}

impl Deserializer<()> {
    /// Creates a builder that can set up the wrapped deserializer.
    pub fn builder() -> DeserializerBuilder<()> {
        DeserializerBuilder::new()
    }
}

impl<'de, D> Deserializer<D>
where
    D: de::Deserializer<'de>,
{
    /// Wraps another [`serde::Deserializer`] and returns the path-aware version.
    #[inline]
    pub fn new(inner_de: D) -> Self {
        Self {
            inner_de: Some(inner_de),
            error_path: ErrorPath::new(None),
            key_modifier: noop,
        }
    }
}

impl<'de, D> From<D> for Deserializer<D>
where
    D: de::Deserializer<'de>,
{
    /// Identical to [`Deserializer::new`]
    #[inline]
    fn from(inner_de: D) -> Self {
        Self::new(inner_de)
    }
}

macro_rules! impl_deserializer_fn {
    ($($fn_name:ident($($arg:ident: $arg_ty:ty),* $(,)?)),* $(,)?) => {
        $(
            fn $fn_name<V>(mut self, $($arg: $arg_ty,)* visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                let inner_de = self.inner_de.take()
                    .expect("inner deserializer has already been taken");

                let root = Track::Root;

                let inner = DeserializerImpl::new(
                    inner_de,
                    &root,
                    &self.error_path,
                    self.key_modifier,
                );

                match inner.$fn_name($($arg,)* visitor) {
                    Ok(ok) => Ok(ok),
                    Err(err) => Err(Error::new(err, self.error_path.take()))
                }
            }
        )*
    };
}

impl<'de, D> de::Deserializer<'de> for Deserializer<D>
where
    D: de::Deserializer<'de>,
{
    type Error = Error<D::Error>;

    impl_deserializer_fn! {
        deserialize_any(),
        deserialize_bool(),
        deserialize_char(),
        deserialize_str(),
        deserialize_string(),
        deserialize_bytes(),
        deserialize_byte_buf(),
        deserialize_option(),
        deserialize_unit(),
        deserialize_map(),
        deserialize_seq(),
        deserialize_identifier(),
        deserialize_ignored_any(),
        deserialize_i8(),
        deserialize_i16(),
        deserialize_i32(),
        deserialize_i64(),
        deserialize_u8(),
        deserialize_u16(),
        deserialize_u32(),
        deserialize_u64(),
        deserialize_f32(),
        deserialize_f64(),
        deserialize_unit_struct(name: &'static str),
        deserialize_newtype_struct(name: &'static str),
        deserialize_tuple(len: usize),
        deserialize_tuple_struct(name: &'static str, len: usize),
        deserialize_struct(name: &'static str, fields: &'static [&'static str]),
        deserialize_enum(name: &'static str, variants: &'static [&'static str]),
    }
}
