use std::fmt;

use gcp_auth_provider::channel::ChannelError;
use protos::bigquery_storage::storage_error::StorageErrorCode;
use protos::bigquery_storage::{AppendRowsResponse, RowError, StorageError};
use tokio::task::JoinError;

use crate::proto::EncodeError;
#[cfg(feature = "read")]
use crate::read::DeserializeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth_provider::Error),
    #[error(transparent)]
    Internal(InternalError),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    RowInsert(#[from] RowInsertErrors),
    #[cfg(feature = "read")]
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Commit(#[from] CommitError),
}

impl From<ChannelError> for Error {
    fn from(value: ChannelError) -> Self {
        match value {
            ChannelError::Auth(auth) => Self::Auth(auth),
            ChannelError::Transport(err) => Self::Transport(err),
        }
    }
}

#[derive(Debug)]
pub struct RowInsertErrors {
    status: protos::rpc::Status,
    row_errors: Vec<RowError>,
}

impl fmt::Display for RowInsertErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();

        match self.row_errors.len() {
            0 => write!(f, "row insert error - {code}: {}", self.status.message),
            count => write!(
                f,
                "row insert error - {code} ({count} row errors): {}",
                self.status.message,
            ),
        }
    }
}

impl std::error::Error for RowInsertErrors {}

impl RowInsertErrors {
    pub(crate) fn new(status: protos::rpc::Status, row_errors: Vec<RowError>) -> Self {
        Self { status, row_errors }
    }

    pub(crate) fn code(&self) -> tonic::Code {
        tonic::Code::from_i32(self.status.code)
    }

    pub(crate) fn from_raw_response(raw: AppendRowsResponse) -> Result<Option<i64>, Error> {
        use protos::bigquery_storage::append_rows_response::Response;

        match raw.response {
            Some(Response::AppendResult(res)) => {
                if !raw.row_errors.is_empty() {
                    tracing::error!(
                        message = "AppendRowsResponse.row_errors is not empty for an Ok result",
                        row_errors = ?raw.row_errors,
                        write_stream = raw.write_stream,
                    );
                }
                Ok(res.offset.map(|int| int.value))
            }
            Some(Response::Error(status)) => {
                Err(Error::RowInsert(Self::new(status, raw.row_errors)))
            }
            None => {
                tracing::error!(
                    message = "AppendRowsResponse.response is None",
                    write_stream = raw.write_stream
                );
                Err(Error::Status(tonic::Status::internal(
                    "AppendRowsResponse.response is None",
                )))
            }
        }
    }
}

impl From<protos::rpc::Status> for Error {
    fn from(mut value: protos::rpc::Status) -> Self {
        let code = tonic::Code::from_i32(value.code);
        match value.details.len() {
            0 => Self::Status(tonic::Status::new(code, value.message)),
            1.. => Self::Status(tonic::Status::with_details(
                code,
                value.message,
                value.details.swap_remove(0).value,
            )),
        }
    }
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
    #[error("more append row responses were recieved than expected")]
    TooManyAppendRowResponses,
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
