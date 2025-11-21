//! Path aware [`Error`] type.
//!
//! [`Deserializer`]: [`super::Deserializer`]

use std::fmt;

use serde::de;
use serde::ser::{self, SerializeMap, SerializeStruct};

use crate::Path;

/// A path-aware deserialization error, containing an error `E` and optional [`Path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error<E> {
    error: E,
    path: Option<Path>,
}

impl<E> From<E> for Error<E> {
    #[inline]
    fn from(error: E) -> Self {
        Self { error, path: None }
    }
}

impl<E> Error<E> {
    /// Serializes this wrapped error as a map, into a given deserializer. Must provide a function
    /// to encode the inner error type as a map, and the expected number of entries to be written.
    pub fn serialize_as_map_with<S, F>(
        &self,
        serializer: S,
        with_fn: F,
        with_fn_len: usize,
    ) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
        F: FnOnce(
            &E,
            &mut <S as ser::Serializer>::SerializeMap,
        )
            -> Result<(), <<S as ser::Serializer>::SerializeMap as ser::SerializeMap>::Error>,
    {
        let mut map_ser = serializer.serialize_map(Some(with_fn_len + 1))?;
        with_fn(&self.error, &mut map_ser)?;
        map_ser.serialize_entry("path", &self.path)?;

        map_ser.end()
    }
}

impl<E> ser::Serialize for Error<E>
where
    E: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use serde_helpers::display_serialize::DisplaySerialize;

        let capacity = if self.path.is_some() { 2 } else { 1 };
        let mut struct_ser = serializer.serialize_struct("Error", capacity)?;

        struct_ser.serialize_field("error", &DisplaySerialize(&self.error))?;

        if let Some(path) = self.path.as_ref() {
            struct_ser.serialize_field("path", &DisplaySerialize(&path))?;
        }

        struct_ser.end()
    }
}

#[cfg(feature = "axum")]
impl<E: fmt::Display> axum::response::IntoResponse for Error<E> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

impl<E> Error<E> {
    pub(super) fn new(error: E, path: Option<Path>) -> Self {
        Self { error, path }
    }

    /// converts the inner error into a type erased [`Box<dyn std:::error::Error>`]
    pub fn into_boxed(self) -> Error<Box<dyn std::error::Error>>
    where
        E: std::error::Error + 'static,
    {
        Error {
            path: self.path,
            error: Box::new(self.error),
        }
    }

    /// converts the inner error into a type erased [`Box<dyn std:::error::Error + Send + Sync>`]
    pub fn into_send_sync_boxed(self) -> Error<Box<dyn std::error::Error + Send + Sync>>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error {
            path: self.path,
            error: Box::new(self.error),
        }
    }

    /// Maps the inner error type to another type.
    #[inline]
    pub fn map_error<E2>(self, map: impl FnOnce(E) -> E2) -> Error<E2> {
        Error {
            error: map(self.error),
            path: self.path,
        }
    }

    /// Maps the inner error type to another type, fallibly.
    #[inline]
    pub fn try_map_error<E2>(
        self,
        try_map: impl FnOnce(E) -> Result<E2, E>,
    ) -> Result<Error<E2>, Self> {
        let Self { error, path } = self;

        match try_map(error) {
            Ok(error) => Ok(Error { error, path }),
            Err(error) => Err(Self { error, path }),
        }
    }

    /// Returns a reference to the inner error.
    pub fn error(&self) -> &E {
        &self.error
    }

    /// Returns an optional reference to the inner path.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref()
    }

    /// Consumes [`Self`], returning the inner error (`E`).
    pub fn into_error(self) -> E {
        self.error
    }

    /// Consumes [`Self`], returning the optional [`Path`].
    pub fn into_path(self) -> Option<Path> {
        self.path
    }

    /// Consumes [`Self`], returning the inner contents as: `([`E`], [`Option<Path>`])`.
    pub fn into_inner(self) -> (E, Option<Path>) {
        (self.error, self.path)
    }
}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if let Some(path) = self.path.as_ref() {
            if !path.is_root() {
                write!(formatter, "at '{path}': ")?;
            }
        }

        write!(formatter, "{}", self.error)
    }
}

impl<E: std::error::Error> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // we cant return `self.error` as a source without requiring a 'static bound,
        // which makes using [`Error`] much more of a pain in other scenarios (usually
        // related to nested deserialization).
        //
        // in order to avoid that, we just use the errors 'source' method instead.
        // that does mean a step in the error chain does get skipped, but Error provides
        // direct access to the error via [Error::error], so it shouldn't be too annoying
        self.error.source()
    }
}

impl<E: de::Error> de::Error for Error<E> {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            error: E::custom(msg),
            path: None,
        }
    }
}

impl<E: ser::Error> ser::Error for Error<E> {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            error: E::custom(msg),
            path: None,
        }
    }
}

/*
#[cfg(feature = "schemars")]
mod schemars_impl {
    use schemars::JsonSchema;
    use schemars::gen::SchemaGenerator;
    use schemars::schema::{InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec};

    impl<E> JsonSchema for super::Error<E>
    where
        E: std::fmt::Display,
    {
        fn schema_name() -> String {
            "PathAwareError".to_owned()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            let mut properties = std::collections::BTreeMap::new();

            properties.insert("error".to_owned(), gen.subschema_for::<String>());
            properties.insert("path".to_owned(), gen.subschema_for::<Option<String>>());

            Schema::Object(SchemaObject {
                instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
                object: Some(Box::new(ObjectValidation {
                    min_properties: Some(1),
                    max_properties: Some(2),
                    properties,
                    required: Some("error".to_owned()).into_iter().collect(),
                    ..Default::default()
                })),
                ..Default::default()
            })
        }
    }
}
*/
