//! Similar to [`DisplaySerialize`], but tailored for [`std::error::Error`] implementing types.

use std::error::Error;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "deepsize", derive(deepsize::DeepSizeOf))]
pub struct DisplayError<E>(pub E);

impl<E> From<E> for DisplayError<E> {
    #[inline]
    fn from(err: E) -> Self {
        Self(err)
    }
}

impl<E> Deref for DisplayError<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for DisplayError<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> Serialize for DisplayError<E>
where
    E: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

impl<E> fmt::Display for DisplayError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        <E as fmt::Display>::fmt(&self.0, formatter)
    }
}

impl<E> Error for DisplayError<E>
where
    E: Error,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }

    #[inline]
    fn description(&self) -> &str {
        // want to pass through all function calls, regardless of deprecation status
        #[allow(deprecated)]
        self.0.description()
    }

    #[inline]
    fn cause(&self) -> Option<&dyn Error> {
        // want to pass through all function calls, regardless of deprecation status
        #[allow(deprecated)]
        self.0.cause()
    }
}
