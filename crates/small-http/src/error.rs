use std::io;

use hyper::client::conn::TrySendError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] http::Error),
}

macro_rules! impl_from_http_sub_error_types {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for Error {
                #[inline]
                fn from(value: $t) -> Self {
                    Self::Http(http::Error::from(value))
                }
            }
        )*
    };
}

impl_from_http_sub_error_types! {
    http::uri::InvalidUri,
    http::uri::InvalidUriParts,
    http::header::InvalidHeaderName,
    http::header::InvalidHeaderValue,
    http::header::MaxSizeReached,
    http::method::InvalidMethod,
    http::status::InvalidStatusCode,
}

impl Error {
    #[inline]
    pub(crate) fn io(
        kind: io::ErrorKind,
        error: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Io(io::Error::new(kind, error))
    }

    #[inline]
    pub(crate) fn io_invalid_input(
        error: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::InvalidInput, error))
    }

    pub fn is_connection_closed(&self) -> bool {
        match self {
            Self::Hyper(err) => err.is_closed() || err.is_incomplete_message(),
            _ => false,
        }
    }
}

impl From<std::convert::Infallible> for Error {
    #[inline]
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

impl<T> From<TrySendError<T>> for Error {
    #[inline]
    fn from(value: TrySendError<T>) -> Self {
        Self::Hyper(value.into_error())
    }
}
