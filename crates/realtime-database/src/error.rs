use std::fmt;
use std::str::Utf8Error;

use thiserror::Error;

use crate::event::EventType;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidEvent(#[from] InvalidEvent),
    #[error(transparent)]
    Auth(#[from] gcp_auth_provider::Error),
    #[error(transparent)]
    RealtimeDatabase(#[from] RealtimeDbError),
    #[error(transparent)]
    Timestamp(#[from] timestamp::Error),
    #[error("Internal task error")]
    InternalTaskError,
    #[error(transparent)]
    DeserializeError(SerdeError),
    #[error(transparent)]
    SerializeError(SerdeError),
}

impl From<timestamp::error::ConvertError> for Error {
    fn from(value: timestamp::error::ConvertError) -> Self {
        Self::Timestamp(value.into())
    }
}

impl Error {
    pub(crate) fn de<E>(de_err: E) -> Self
    where
        E: Into<SerdeError>,
    {
        Self::DeserializeError(de_err.into())
    }
}

#[derive(Debug, Error)]
pub enum SerdeError {
    #[error("{0}")]
    Error(#[from] serde_json::Error),
    #[error("{0}")]
    PathAwareError(#[from] path_aware_serde::Error<serde_json::Error>),
}

impl From<tokio::task::JoinError> for Error {
    fn from(task_err: tokio::task::JoinError) -> Self {
        tracing::error!("task join error: {}", task_err);
        Self::InternalTaskError
    }
}

impl From<Utf8Error> for Error {
    fn from(inner: Utf8Error) -> Self {
        Error::InvalidEvent(InvalidEvent::InvalidUTF8(inner))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RealtimeDbError {
    error: String,
    #[serde(default, deserialize_with = "deserialize_none")]
    reqwest_error: Option<reqwest::Error>,
}

impl RealtimeDbError {
    pub(crate) fn with_reqwest_error(self, error: reqwest::Error) -> Self {
        Self {
            error: self.error,
            reqwest_error: Some(error),
        }
    }
}

fn deserialize_none<'de, D, T>(_deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(None)
}

impl fmt::Display for RealtimeDbError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.reqwest_error.as_ref() {
            Some(reqwest_err) => write!(formatter, "{}: {}", reqwest_err, self.error),
            None => formatter.write_str(&self.error),
        }
    }
}

impl std::error::Error for RealtimeDbError {}

#[derive(Debug, Error)]
pub enum InvalidEvent {
    #[error("Event is missing the event type tag")]
    MissingEvent,
    #[error("Unknown event type '{0}'")]
    UnknownEventType(#[from] UnknownEventType),
    #[error("{0}")]
    InvalidUTF8(#[from] Utf8Error),
    #[error("Found event type {0}, but no data")]
    MissingEventData(#[from] MissingEventData),
    #[error("event payload invalid: {0}")]
    InvalidDataPayload(#[from] serde_json::Error),
}

impl InvalidEvent {
    pub(crate) fn unknown_event_type<S>(messg: S) -> Self
    where
        S: Into<String>,
    {
        Self::UnknownEventType(UnknownEventType(messg.into()))
    }

    pub(crate) fn missing_data(event_type: EventType) -> Self {
        Self::MissingEventData(MissingEventData(event_type))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEventType(String);

impl fmt::Display for UnknownEventType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0.as_str())
    }
}

impl std::error::Error for UnknownEventType {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingEventData(EventType);

impl fmt::Display for MissingEventData {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0.as_str())
    }
}

impl std::error::Error for MissingEventData {}
