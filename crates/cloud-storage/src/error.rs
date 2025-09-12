use crate::read::InvalidReadBounds;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth_channel::Error),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    InvalidReadBounds(#[from] InvalidReadBounds),
    #[error(transparent)]
    DataError(#[from] DataError),
    #[error("internal error: {0}")]
    Internal(Box<str>),
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::internal(format!("internal task panic: {value}"))
    }
}

impl Error {
    pub(crate) fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(String::into_boxed_str(msg.into()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error(transparent)]
    Crc32c(#[from] DataErrorKind<u32>),
    #[error(transparent)]
    Md5(#[from] DataErrorKind<md5::Digest>),
}

impl DataError {
    pub(crate) fn crc32c(expected: u32, computed: u32) -> Self {
        Self::Crc32c(DataErrorKind {
            kind: "crc32c",
            expected,
            computed,
        })
    }

    pub(crate) fn md5(expected: md5::Digest, computed: md5::Digest) -> Self {
        Self::Md5(DataErrorKind {
            kind: "md5",
            expected,
            computed,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{kind} mismatch: expected {expected:x}, computed {computed:x}")]
pub struct DataErrorKind<D> {
    kind: &'static str,
    expected: D,
    computed: D,
}
