use std::fmt;

use protos::bigquery_storage::StorageError;
use protos::bigquery_storage::storage_error::StorageErrorCode;

#[cfg(any(feature = "storage-write", feature = "storage-read"))]
use super::storage::proto::{EncodeError, FieldPair};
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth::Error),
    // #[cfg(any(feature = "storage-read", feature = "storage-write"))]
    #[error(transparent)]
    Transport(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[cfg(any(feature = "storage-read", feature = "storage-write"))]
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error("Arrow is not a supported format")]
    ArrowNotSupported,
    #[cfg(any(feature = "storage-read", feature = "storage-write"))]
    #[error(transparent)]
    Avro(#[from] apache_avro::Error),
    #[error("internal error")]
    InternalError,
    #[error(transparent)]
    InvalidTimestamp(#[from] timestamp::Error),
    #[error("{0}")]
    Misc(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(any(feature = "rest", feature = "storage-write"))]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("at '{path}': {error}")]
    PathAwareError {
        path: path_aware_serde::Path,
        error: Box<Error>,
    },
    #[error(transparent)]
    CommitError(#[from] CommitError),
    #[error("created write session returned no schema, cannot serialize data without it")]
    NoSchemaReturned,
    #[cfg(feature = "storage-write")]
    #[error(transparent)]
    Encode(#[from] EncodeError),
}

impl Error {
    #[cfg(any(feature = "storage-read", feature = "storage-write"))]
    pub(crate) fn try_from_raw_status(mut status: protos::rpc::Status) -> Result<(), Self> {
        use std::fmt::Write;

        match tonic::Code::from_i32(status.code) {
            tonic::Code::Ok => Ok(()),
            other_code => {
                if !status.details.is_empty() {
                    status.message.push_str("\n- Misc error details:\n");
                }

                for item in status.details {
                    status.message.push('\n');
                    status.message.push_str(item.type_url.as_str());
                    status.message.push('\n');

                    let mut buf = bytes::Bytes::from(item.value);

                    while !buf.is_empty() {
                        if let Ok(ok) = FieldPair::from_buf(&mut buf) {
                            write!(&mut status.message, "{ok:?}").expect("");
                        }
                    }
                }

                Err(Error::Status(tonic::Status::new(
                    other_code,
                    status.message,
                )))
            }
        }
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

impl From<path_aware_serde::Error<Error>> for Error {
    fn from(path_err: path_aware_serde::Error<Error>) -> Self {
        let (error, path_opt) = path_err.into_inner();

        match path_opt {
            Some(path) => Self::PathAwareError {
                path,
                error: Box::new(error),
            },
            None => error,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommitError {
    Many(Vec<StorageError>),
    One(StorageError),
}

impl fmt::Display for CommitError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Many(many) => formatter
                .debug_list()
                .entries(many.iter().map(DisplayStorageError))
                .finish(),
            Self::One(err) => write!(formatter, "{}", DisplayStorageError(err)),
        }
    }
}

impl std::error::Error for CommitError {}

pub struct DisplayStorageError<'a>(&'a StorageError);

impl fmt::Debug for DisplayStorageError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

fn parse_error_code(code: i32) -> Option<StorageErrorCode> {
    match code {
        c if c == StorageErrorCode::Unspecified as i32 => Some(StorageErrorCode::Unspecified),
        c if c == StorageErrorCode::TableNotFound as i32 => Some(StorageErrorCode::TableNotFound),
        c if c == StorageErrorCode::StreamAlreadyCommitted as i32 => {
            Some(StorageErrorCode::StreamAlreadyCommitted)
        }
        c if c == StorageErrorCode::StreamNotFound as i32 => Some(StorageErrorCode::StreamNotFound),
        c if c == StorageErrorCode::InvalidStreamType as i32 => {
            Some(StorageErrorCode::InvalidStreamType)
        }
        c if c == StorageErrorCode::InvalidStreamState as i32 => {
            Some(StorageErrorCode::InvalidStreamState)
        }
        c if c == StorageErrorCode::StreamFinalized as i32 => {
            Some(StorageErrorCode::StreamFinalized)
        }
        c if c == StorageErrorCode::SchemaMismatchExtraFields as i32 => {
            Some(StorageErrorCode::SchemaMismatchExtraFields)
        }
        c if c == StorageErrorCode::OffsetAlreadyExists as i32 => {
            Some(StorageErrorCode::OffsetAlreadyExists)
        }
        c if c == StorageErrorCode::OffsetOutOfRange as i32 => {
            Some(StorageErrorCode::OffsetOutOfRange)
        }
        _ => None,
    }
}

impl fmt::Display for DisplayStorageError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if let Some(code) = parse_error_code(self.0.code) {
            write!(formatter, "Error type {code:?}: ")?;
        } else {
            write!(formatter, "Error code {}: ", self.0.code)?;
        }

        write!(
            formatter,
            "Entity: {}, Error: {}",
            self.0.entity, self.0.error_message
        )
    }
}

impl CommitError {
    pub fn from_raw_errors(mut errors: Vec<StorageError>) -> Result<(), Self> {
        if errors.is_empty() {
            return Ok(());
        }

        match errors.len() {
            0 => Ok(()),
            1 => Err(Self::One(errors.pop().unwrap())),
            _ => Err(Self::Many(errors)),
        }
    }

    pub fn num_errors(&self) -> usize {
        match self {
            Self::Many(many) => many.len(),
            Self::One(_) => 1,
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Misc(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Misc(msg.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        error!("internal task error: {error}");
        Self::InternalError
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(send_err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        error!("internal task error: {send_err}");
        Self::InternalError
    }
}

#[cfg(any(feature = "storage-read", feature = "storage-write"))]
impl From<tonic::metadata::errors::InvalidMetadataValue> for Error {
    fn from(err: tonic::metadata::errors::InvalidMetadataValue) -> Self {
        error!("metadata value error: {err}");
        Self::InternalError
    }
}

impl From<gcp_auth_channel::Error> for Error {
    fn from(err: gcp_auth_channel::Error) -> Self {
        match err {
            gcp_auth_channel::Error::InvalidHeaderName(err) => Error::InvalidHeaderName(err),
            gcp_auth_channel::Error::InvalidHeaderValue(err) => Error::InvalidHeaderValue(err),
            gcp_auth_channel::Error::Auth(err) => Error::Auth(err),
            // #[cfg(any(feature = "storage-read", feature = "storage-write"))]
            gcp_auth_channel::Error::Transport(err) => Error::Transport(err),
        }
    }
}

#[cfg(any(feature = "storage-read", feature = "storage-write"))]
impl From<tonic::transport::Error> for Error {
    fn from(value: tonic::transport::Error) -> Self {
        Self::Transport(Box::new(value))
    }
}
