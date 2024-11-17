use std::fmt;

use protos::bigquery_storage::StorageError;
use protos::bigquery_storage::storage_error::StorageErrorCode;
use tokio::task::JoinError;

use crate::proto::EncodeError;
#[cfg(feature = "read")]
use crate::read::DeserializeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth_channel::Error),
    #[error(transparent)]
    Internal(InternalError),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[cfg(feature = "read")]
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Commit(#[from] CommitError),
}

#[cfg(feature = "read")]
impl From<apache_avro::Error> for Error {
    fn from(value: apache_avro::Error) -> Self {
        Self::Deserialize(DeserializeError::from(value))
    }
}

impl<E> From<E> for Error
where
    InternalError: From<E>,
{
    fn from(value: E) -> Self {
        Self::Internal(InternalError::from(value))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error(transparent)]
    Task(#[from] JoinError),
    #[error(transparent)]
    InvalidHeader(#[from] http::header::InvalidHeaderValue),
    #[error("reading via arrow is not yet supported")]
    ArrowNotSupported,
    #[error("no schema was returned to deserialize from")]
    NoSchemaReturned,
}

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

impl Error {
    #[cfg(feature = "write")]
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
                        if let Ok(ok) = crate::proto::FieldPair::from_buf(&mut buf) {
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

impl fmt::Display for DisplayStorageError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match StorageErrorCode::try_from(self.0.code) {
            Ok(code) => write!(formatter, "Error type {code:?}: ")?,
            Err(_) => write!(formatter, "Error code {}: ", self.0.code)?,
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
