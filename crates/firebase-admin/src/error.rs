use std::borrow::Cow;

use bytes::Bytes;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Status(#[from] StatusError),
    #[error(transparent)]
    Json(#[from] path_aware_serde::Error<serde_json::Error>),
    #[error(transparent)]
    ValidateToken(#[from] crate::auth::ValidateTokenError),
    #[error(transparent)]
    Auth(#[from] gcp_auth_provider::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub struct StatusError {
    uri: reqwest::Url,
    status: reqwest::StatusCode,
    kind: StatusErrorKind,
}

impl StatusError {
    pub(crate) fn new_from(uri: reqwest::Url, status: StatusCode, bytes: Bytes) -> Self {
        debug_assert!(
            !status.is_success(),
            "StatusError::new_from called on a successful StatusCode"
        );

        Self {
            uri,
            status,
            kind: StatusErrorKind::from_bytes(bytes),
        }
    }
}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uri_str = self.uri.as_str();
        let uri_minus_qp = uri_str
            .split_once('?')
            .map(|(base, _)| base)
            .unwrap_or(uri_str);

        write!(f, "{uri_minus_qp} - {}", self.status.as_u16())?;

        if let Some(msg) = self.kind.try_extract_message() {
            write!(f, ": {msg}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum StatusErrorKind {
    Empty,
    Raw(bytes::Bytes),
    Json(serde_json::Map<String, serde_json::Value>),
    Array(Vec<serde_json::Value>),
}

impl StatusErrorKind {
    fn from_bytes(bytes: Bytes) -> Self {
        if bytes.is_empty() {
            return Self::Empty;
        }

        if bytes.starts_with(b"{") {
            if let Ok(json) = serde_json::from_slice(&bytes) {
                return Self::Json(json);
            }
        }

        if bytes.starts_with(b"[") {
            if let Ok(json) = serde_json::from_slice(&bytes) {
                return Self::Array(json);
            }
        }

        Self::Raw(bytes)
    }

    fn try_extract_message(&self) -> Option<&str> {
        fn try_convert_bytes_to_str(bytes: &[u8]) -> Option<&str> {
            let s = match std::str::from_utf8(bytes) {
                Ok(s) => s,
                Err(err) if err.valid_up_to() == 0 => return None,
                Err(err) => std::str::from_utf8(&bytes[..err.valid_up_to()]).unwrap(),
            };

            let s = s.trim();
            s.is_empty().then_some(s)
        }

        fn try_extract_from_value(value: &serde_json::Value) -> Option<&str> {
            match value {
                serde_json::Value::String(s) => Some(s.as_str()),
                serde_json::Value::Object(map) => try_extract_from_map(map),
                _ => None,
            }
        }

        fn try_extract_from_map(map: &serde_json::Map<String, serde_json::Value>) -> Option<&str> {
            const CANDIDATE_KEYS: &[&str] = &["error", "message", "reason"];

            let mut candidate = None::<&str>;

            for (key, value) in map.iter() {
                if CANDIDATE_KEYS
                    .iter()
                    .any(|cand| key.eq_ignore_ascii_case(cand))
                {
                    match (try_extract_from_value(value), candidate) {
                        (Some(msg), Some(existing)) if existing.len() < msg.len() => {
                            candidate = Some(msg);
                        }
                        (Some(message), None) => candidate = Some(message),
                        _ => (),
                    }
                }
            }

            candidate
        }

        match self {
            Self::Empty => None,
            Self::Raw(bytes) => try_convert_bytes_to_str(bytes),
            Self::Json(map) => try_extract_from_map(map),
            Self::Array(array) => array.iter().find_map(|value| match value {
                serde_json::Value::String(s) => Some(s.as_str()),
                serde_json::Value::Object(map) => try_extract_from_map(map),
                _ => None,
            }),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::ValidateToken(value.into())
    }
}

impl Error {
    pub fn to_response_parts(&self) -> (StatusCode, Cow<'static, str>) {
        match self {
            Self::ValidateToken(auth) => auth.to_response_parts(),
            Self::Status(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Owned(error.to_string()),
            ),
            Self::Io(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Owned(error.to_string()),
            ),
            Self::Auth(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Owned(error.to_string()),
            ),
            Self::Json(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Owned(error.to_string()),
            ),
            Self::Reqwest(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Cow::Owned(error.to_string()),
            ),
        }
    }
}
