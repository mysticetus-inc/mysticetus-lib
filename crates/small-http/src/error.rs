use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
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
}
