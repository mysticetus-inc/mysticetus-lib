//! Wrapper to serialize a type using it's [`fmt::Display`] impl.

use std::fmt;

use serde::Serialize;

pub struct DisplaySerialize<T>(pub T);

impl<T> Serialize for DisplaySerialize<T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}
