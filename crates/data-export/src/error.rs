use std::fmt;

#[derive(Debug, thiserror::Error)]
enum Inner {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Fmt(#[from] fmt::Error),
    // #[error(transparent)]
    // Xlsx(#[from] xlsxwriter::XlsxError),
    #[error(transparent)]
    RustXlsx(#[from] rust_xlsxwriter::XlsxError),
    #[error("{0}")]
    Misc(String),
}

pub struct Error {
    inner: Inner,
}

impl Error {
    #[inline]
    pub fn misc<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            inner: Inner::Misc(message.into()),
        }
    }

    #[inline]
    pub fn misc_display<S>(message: S) -> Self
    where
        S: fmt::Display,
    {
        Self {
            inner: Inner::Misc(message.to_string()),
        }
    }
}

impl<T> From<T> for Error
where
    T: Into<Inner>,
{
    #[inline]
    fn from(inner: T) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Inner as fmt::Debug>::fmt(&self.inner, f)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Inner as fmt::Display>::fmt(&self.inner, f)
    }
}

impl std::error::Error for Error {}
