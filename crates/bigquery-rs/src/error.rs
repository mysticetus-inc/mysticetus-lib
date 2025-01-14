use crate::resources::ErrorProto;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] gcp_auth_channel::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error(transparent)]
    Json(#[from] path_aware_serde::Error<serde_json::Error>),
    #[error("{main}")]
    JobError {
        main: ErrorProto,
        misc: Vec<ErrorProto>,
    },
}

impl From<ErrorProto> for Error {
    fn from(value: ErrorProto) -> Self {
        Self::JobError {
            main: value,
            misc: vec![],
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("expected valid value for `{ty}.{field}`, not {value:?}")]
pub struct MissingField {
    ty: &'static str,
    field: &'static str,
    value: Box<dyn std::fmt::Debug + Send + Sync>,
}

impl MissingField {
    pub fn new<T>(
        field: &'static str,
        value: impl std::fmt::Debug + Send + Sync + 'static,
    ) -> Self {
        Self {
            field,
            ty: std::any::type_name::<T>(),
            value: Box::from(value),
        }
    }
}

impl Error {
    pub fn missing_field<T>(
        field: &'static str,
        value: impl std::fmt::Debug + Send + Sync + 'static,
    ) -> Self {
        Self::MissingField(MissingField::new::<T>(field, value))
    }

    pub fn is_not_found(&self) -> bool {
        match self {
            Self::Reqwest(req) => matches!(req.status(), Some(reqwest::StatusCode::NOT_FOUND)),
            Self::JobError { main, .. } => main.is_not_found(),
            _ => false,
        }
    }
}

/*
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[cfg(any(feature = "storage-read", feature = "storage-write"))]
    #[error(transparent)]
    Avro(#[from] apache_avro::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] Box<Error>),
}

impl From<apache_avro::Error> for Error {
    fn from(error: apache_avro::Error) -> Self {
        Self::PathAwareError {
            path: Default::default(),
            error: FormatError::Avro(error),
        }
    }
}

impl From<FormatError> for Error {
    fn from(error: FormatError) -> Self {
        Self::PathAwareError {
            path: Default::default(),
            error,
        }
    }
}

impl From<Error> for FormatError {
    fn from(value: Error) -> Self {
        Self::Other(Box::new(value))
    }
}

impl<E> From<path_aware_serde::Error<E>> for Error
where
    FormatError: From<E>,
{
    fn from(value: path_aware_serde::Error<E>) -> Self {
        let (error, path) = value.into_inner();
        Self::PathAwareError {
            path: path.unwrap_or_default(),
            error: FormatError::from(error),
        }
    }
}
*/

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

/*
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
*/
