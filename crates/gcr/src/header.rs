//!
use std::fmt;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::HeaderValue;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

/// The admin header.
///
/// This is a [`str`], since [`HeaderName`] contains interior mutability
/// (deep in [`bytes`]: https://docs.rs/bytes/1.3.0/src/bytes/bytes.rs.html#104),
/// and while it's unlikly to ever be mutated in practice, a static string
/// also implements all the same traits that [`HeaderName`] does, so it's interchangable
/// enough.
pub const ADMIN_FLAG_HEADER: &str = "x-mysti-admin";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdminHeader<T = HeaderValue>(pub T);

pub trait FromHeader: Sized + fmt::Debug {
    type Rejection: IntoResponse;

    /// Parse/validate the value from the raw header value.
    fn from_header(value: HeaderValue) -> Result<Self, Self::Rejection>;

    /// Render the parsed/validated value for logging. Shares the same
    /// signature as [`fmt::Debug`] and [`fmt::Display`]. Used by
    /// both [`fmt`] trait impls on [`AdminHeader`].
    ///
    /// The defualt implementation defers to [`fmt::Debug`],
    /// but it might be desirable to have a one off impl.
    fn log_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<T: FromHeader> fmt::Debug for AdminHeader<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AdminHeader").field(&self.0).finish()
    }
}

impl<T: FromHeader> fmt::Display for AdminHeader<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.log_fmt(f)
    }
}

impl FromHeader for HeaderValue {
    type Rejection = std::convert::Infallible;

    fn from_header(value: HeaderValue) -> Result<Self, Self::Rejection> {
        Ok(value)
    }
}

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for AdminHeader<T>
where
    T: FromHeader,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.remove(ADMIN_FLAG_HEADER) {
            Some(header) => match T::from_header(header) {
                Ok(ok) => Ok(Self(ok)),
                Err(error) => Err(error.into_response()),
            },
            None => {
                error!("missing admin header");
                Err(StatusCode::BAD_REQUEST.into_response())
            }
        }
    }
}