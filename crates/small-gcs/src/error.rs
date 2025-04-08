use std::collections::HashMap;
use std::fmt;

use bytes::Bytes;
use reqwest::StatusCode;
use reqwest::header::InvalidHeaderValue;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NotFound(ErrorPayload),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    BadRequest(ErrorPayload),
    #[error(transparent)]
    Auth(#[from] gcp_auth_channel::Error),
    #[error(transparent)]
    PreconditionFailed(ErrorPayload),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// errors that indicate an issue with this API wrapper. Should be alerted
    /// if they ever occur.
    #[error(transparent)]
    Internal(#[from] InternalError),
}

impl Error {
    pub(crate) fn missing_resource() -> Self {
        Self::Internal(InternalError {
            repr: InternalImpl::MissingResource,
        })
    }
}

/// Validates that the response is a 2XX status and returns it back as [`Ok`],
/// or consumes the response and builds the appropriate [`Error`].
pub(super) async fn validate_response(resp: reqwest::Response) -> Result<reqwest::Response, Error> {
    let status = resp.status();

    macro_rules! extract_error {
        (Internal: $status:expr, $resp:expr) => {{
            let bytes = $resp.bytes().await?;
            let payload = ErrorPayload::from_raw_parts($status, bytes)?;
            Error::Internal(InternalError {
                repr: InternalImpl::Google(payload),
            })
        }};
        ($kind:ident : $status:expr, $resp:expr) => {{
            let bytes = $resp.bytes().await?;
            let payload = ErrorPayload::from_raw_parts($status, bytes)?;
            Error::$kind(payload)
        }};
    }

    match status.as_u16() {
        404 => Err(extract_error!(NotFound: status, resp)),
        405 | 411 => Err(extract_error!(Internal: status, resp)),
        412 => Err(extract_error!(PreconditionFailed: status, resp)),
        400..=499 => Err(extract_error!(BadRequest: status, resp)),
        _ => resp.error_for_status().map_err(Error::Reqwest),
    }
}

#[repr(transparent)]
pub struct InternalError {
    repr: InternalImpl,
}

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_map();

        match &self.repr {
            InternalImpl::Json(err) => dbg.entry(&"kind", &"json").entry(&"error", &err).finish(),
            InternalImpl::InvalidHeader(err) => dbg
                .entry(&"kind", &"invalid_header")
                .entry(&"error", &err)
                .finish(),
            InternalImpl::Google(err) => {
                dbg.entry(&"kind", &"google").entry(&"error", &err).finish()
            }
            InternalImpl::MissingResource => dbg.entry(&"kind", &"missing_resource").finish(),
        }
    }
}

impl From<InternalImpl> for Error {
    fn from(repr: InternalImpl) -> Self {
        Self::Internal(InternalError { repr })
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::Internal(InternalError {
            repr: InternalImpl::InvalidHeader(value),
        })
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.repr.fmt(f)
    }
}

impl std::error::Error for InternalError {}

#[derive(Debug, thiserror::Error)]
enum InternalImpl {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error(transparent)]
    Google(#[from] ErrorPayload),
    #[error("missing expected 'resource' field in response")]
    MissingResource,
}

impl ErrorPayload {
    fn from_raw_parts(status: StatusCode, payload: Bytes) -> Result<Self, InternalError> {
        // use the leading non-whitespace byte to hint at what kind of payload it might be.
        let leading_byte = payload.trim_ascii_start().first().copied();

        match leading_byte {
            // likely a nested google-format error message
            Some(b'{') => match serde_json::from_slice::<NestedPayload>(&payload) {
                Ok(NestedPayload { error }) => Ok(error),
                Err(error) => Err(InternalError { repr: error.into() }),
            },
            // maybe an array of the error details? if not, dont throw an error and just treat as
            // text, since this should in theory never happen.
            Some(b'[') => match serde_json::from_slice::<Vec<ErrorDetail>>(&payload) {
                Ok(errors) => Ok(ErrorPayload::from_errors(status, errors)),
                Err(_) => Ok(ErrorPayload::from_message(status.as_u16(), payload)),
            },
            // if it's another byte, we can assume it's likely text
            Some(_) => Ok(ErrorPayload::from_message(status.as_u16(), payload)),
            // if the payload is empty/just whitespace, just use the status string itself
            // as the message, so we have something non-empty.
            None => Ok(ErrorPayload::from_status(status)),
        }
    }

    fn from_errors(status: StatusCode, errors: Vec<ErrorDetail>) -> Self {
        // pull out the first error message if we can, otherwise fall back to the
        // status-only constructor.
        match errors.first() {
            Some(detail) => Self {
                code: status.as_u16(),
                message: detail.message.clone(),
                errors,
            },
            None => Self::from_status(status),
        }
    }

    fn from_status(status: StatusCode) -> Self {
        Self {
            code: status.as_u16(),
            message: String::from(status.as_str()),
            errors: vec![],
        }
    }

    fn from_message(code: u16, message: Bytes) -> Self {
        Self {
            code,
            message: String::from_utf8_lossy(&message).into_owned(),
            errors: vec![],
        }
    }
}

/// Generic error payloads sent back from Google.
#[derive(Debug, serde::Deserialize)]
pub struct ErrorPayload {
    code: u16,
    message: String,
    #[serde(default)]
    errors: Vec<ErrorDetail>,
}

impl ErrorPayload {
    pub const fn code(&self) -> u16 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn errors(&self) -> &[ErrorDetail] {
        &self.errors
    }

    pub fn len(&self) -> usize {
        self.errors.len().max(1)
    }
}

#[derive(serde::Deserialize)]
pub struct ErrorDetail {
    message: String,
    reason: String,
    #[serde(default, flatten)]
    misc: HashMap<String, Value>,
}

pub struct ErrorDetailIter<'a> {
    misc: std::collections::hash_map::Iter<'a, String, Value>,
}

impl<'a> Iterator for ErrorDetailIter<'a> {
    type Item = (&'a str, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        let (name, value) = self.misc.next()?;
        Some((name.as_str(), value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.misc.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for ErrorDetailIter<'_> {}

impl fmt::Debug for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("ErrorDetail");

        dbg.field("message", &self.message);
        dbg.field("reason", &self.reason);

        for (name, value) in self.misc.iter() {
            dbg.field(&name, &ValueDbg(value));
        }

        dbg.finish()
    }
}

/// Helper type to strip the enum info from the debug repr.
struct ValueDbg<'a>(&'a Value);

impl fmt::Debug for ValueDbg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => b.fmt(f),
            Value::Number(n) => {
                if let Some(float) = n.as_f64() {
                    float.fmt(f)
                } else if let Some(uint) = n.as_u64() {
                    uint.fmt(f)
                } else if let Some(int) = n.as_i64() {
                    int.fmt(f)
                } else {
                    n.fmt(f)
                }
            }
            Value::String(s) => s.fmt(f),
            Value::Array(arr) => f.debug_list().entries(arr.iter().map(Self)).finish(),
            Value::Object(obj) => f
                .debug_map()
                .entries(obj.iter().map(|(key, val)| (key, Self(val))))
                .finish(),
        }
    }
}

impl<'a> IntoIterator for &'a ErrorDetail {
    type Item = (&'a str, &'a Value);
    type IntoIter = ErrorDetailIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.details()
    }
}

impl ErrorDetail {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn details(&self) -> ErrorDetailIter<'_> {
        ErrorDetailIter {
            misc: self.misc.iter(),
        }
    }
}

impl fmt::Display for ErrorPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = itoa::Buffer::new();

        f.write_str("error code ")?;
        f.write_str(buf.format(self.code))?;
        f.write_str(": ")?;
        f.write_str(&self.message)?;

        // if there's more errors than just the 1, say so
        if self.errors.len() > 1 {
            f.write_str(" and ")?;
            f.write_str(buf.format(self.errors.len() - 1))?;
            f.write_str(" others...")
        } else {
            Ok(())
        }
    }
}

impl std::error::Error for ErrorPayload {}

#[derive(serde::Deserialize)]
struct NestedPayload {
    error: ErrorPayload,
}
