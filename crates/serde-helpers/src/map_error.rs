//! A error wrapper that serializes as a map for easy json-like error messages.
use std::error::Error;
use std::fmt;

use serde::Serialize;
use serde::ser::SerializeMap;

use super::display_serialize::DisplaySerialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapError<E, K = &'static str> {
    error: E,
    kind: K,
}

impl<E, K> MapError<E, K> {
    #[inline]
    pub const fn new(kind: K, error: E) -> Self {
        Self { kind, error }
    }
}

impl<E, K> From<(K, E)> for MapError<E, K> {
    fn from(tup: (K, E)) -> Self {
        Self::new(tup.0, tup.1)
    }
}

impl<E> From<E> for MapError<E> {
    fn from(error: E) -> Self {
        Self::new(std::any::type_name::<E>(), error)
    }
}

impl<E, K> fmt::Display for MapError<E, K>
where
    E: fmt::Display,
    K: AsRef<str>,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_ref(), self.error)
    }
}

impl<E, K> Error for MapError<E, K>
where
    E: Error,
    K: AsRef<str> + fmt::Debug,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }

    #[inline]
    fn description(&self) -> &str {
        // want to pass through all function calls, regardless of deprecation status
        #[allow(deprecated)]
        self.error.description()
    }

    #[inline]
    fn cause(&self) -> Option<&dyn Error> {
        // want to pass through all function calls, regardless of deprecation status
        #[allow(deprecated)]
        self.error.cause()
    }
}

impl<E, K> Serialize for MapError<E, K>
where
    E: fmt::Display,
    K: AsRef<str>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(2))?;

        map_ser.serialize_entry("kind", self.kind.as_ref())?;
        map_ser.serialize_entry("error", &DisplaySerialize(&self.error))?;

        map_ser.end()
    }
}
