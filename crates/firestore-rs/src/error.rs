//! Defines error types encountered by this crate.

use std::fmt;

use path_aware_serde::{Error as PathAwareError, Path};

/// Generic error types encountered in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Auth error: {0}")]
    Auth(#[from] gcp_auth_channel::Error),
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("Status: {0}")]
    Status(#[from] tonic::Status),
    #[error("{0}")]
    Convert(ConvertError),
    #[error(
        "document size of {size} is over the limit of {} (document id: {document_id})",
        crate::doc::MAX_DOCUMENT_SIZE
    )]
    OverSizeLimit { document_id: String, size: usize },
    #[error("internal error: {0}")]
    Internal(&'static str),
    #[error("{code}: {message}")]
    RpcError {
        code: tonic::Code,
        message: String,
        details: Option<Vec<protos::protobuf::Any>>,
    },
    #[error("{0}")]
    Many(Errors),
}

// manual From Error/Errors impls to avoid nesting if we already have an instance of many errors
impl From<Error> for Errors {
    fn from(err: Error) -> Self {
        match err {
            Error::Many(errors) => errors,
            _ => Errors { errors: vec![err] },
        }
    }
}

// if we only have 1 error inside errors, just use it by itself and avoid nesting
impl From<Errors> for Error {
    fn from(mut errors: Errors) -> Self {
        if errors.len() == 1 {
            errors.errors.pop().unwrap()
        } else {
            Self::Many(errors)
        }
    }
}

#[derive(Debug)]
pub struct Errors {
    errors: Vec<Error>,
}

impl Errors {
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    // constructing Errors should only ever happen with a non-0 number of errors, so this
    // should in theory always return false.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.errors.first() {
            Some(first) => write!(
                formatter,
                "found {} errors: {}...",
                self.errors.len(),
                first
            ),
            _ => write!(formatter, "no errors found, internal error"),
        }
    }
}

impl std::error::Error for Errors {}

impl std::ops::Deref for Errors {
    type Target = [Error];

    fn deref(&self) -> &Self::Target {
        self.errors.as_slice()
    }
}

impl IntoIterator for Errors {
    type Item = Error;
    type IntoIter = std::vec::IntoIter<Error>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl Error {
    pub(crate) fn check_rpc_status(status: protos::rpc::Status) -> Result<(), Self> {
        let code = tonic::Code::from_i32(status.code);

        if code == tonic::Code::Ok {
            return Ok(());
        }

        let details = match status.details.is_empty() {
            true => None,
            _ => Some(status.details),
        };

        Err(Self::RpcError {
            code,
            message: status.message,
            details,
        })
    }

    pub(crate) fn check_many_rpc_statuses(statuses: Vec<protos::rpc::Status>) -> Result<(), Self> {
        if statuses.is_empty() {
            return Ok(());
        }

        let mut errors: Vec<Self> = statuses
            .into_iter()
            .filter_map(|status| Self::check_rpc_status(status).err())
            .collect();

        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.remove(0)),
            _ => Err(Error::Many(Errors { errors })),
        }
    }
}

impl<C> From<C> for Error
where
    C: Into<ConvertError>,
{
    fn from(conv_err: C) -> Self {
        Self::Convert(conv_err.into())
    }
}

/// Specific error type encountered in document serialization/deserialization.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("Serialization: {error}")]
    Serialize { error: StringErr },
    #[error("Serialization at: '{path}': {error}")]
    SerializeWithPath { path: Path, error: StringErr },
    #[error("Deserialization: {error}")]
    Deserialize { error: StringErr },
    #[error("Deserialization at: '{path}': {error}")]
    DeserializeWithPath { path: Path, error: StringErr },
}

impl ConvertError {
    pub(crate) fn ser<S>(msg: S) -> Self
    where
        S: fmt::Display,
    {
        Self::Serialize {
            error: StringErr(msg.to_string()),
        }
    }

    pub(crate) fn de<S>(msg: S) -> Self
    where
        S: fmt::Display,
    {
        Self::Deserialize {
            error: StringErr(msg.to_string()),
        }
    }

    pub(crate) fn from_path_aware(wrapped_error: PathAwareError<Self>) -> Self {
        let (conv_error, path) = match wrapped_error.into_inner() {
            (err, Some(path)) => (err, path),
            (err, None) => return err,
        };

        match conv_error {
            Self::Serialize { error } => Self::SerializeWithPath { path, error },
            Self::SerializeWithPath { error, .. } => Self::SerializeWithPath { path, error },
            Self::Deserialize { error } => Self::DeserializeWithPath { path, error },
            Self::DeserializeWithPath { error, .. } => Self::DeserializeWithPath { path, error },
        }
    }
}

#[derive(Clone)]
pub struct StringErr(String);

impl From<String> for StringErr {
    fn from(string: String) -> Self {
        Self(string)
    }
}

impl From<&str> for StringErr {
    fn from(string: &str) -> Self {
        Self(string.to_owned())
    }
}

impl From<serde_json::Error> for StringErr {
    fn from(json_err: serde_json::Error) -> Self {
        Self(json_err.to_string())
    }
}

impl fmt::Debug for StringErr {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl fmt::Display for StringErr {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::error::Error for StringErr {}

impl serde::ser::Error for ConvertError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Serialize {
            error: msg.to_string().into(),
        }
    }
}

impl serde::de::Error for ConvertError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Deserialize {
            error: msg.to_string().into(),
        }
    }
}
