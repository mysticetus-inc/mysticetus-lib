use aws_lc_rs::error::{KeyRejected, Unspecified};
use bytes::Bytes;
use http::{HeaderMap, StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Hyper(#[from] HyperError),
    #[error(transparent)]
    Json(#[from] path_aware_serde::Error<serde_json::Error>),
    #[error(transparent)]
    RsaError(#[from] RsaError),
    #[error(transparent)]
    Response(#[from] ResponseError),
    #[error("no authentication provider was found")]
    NoProviderFound,
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RsaError {
    #[error(transparent)]
    KeyRejected(#[from] KeyRejected),
    #[error(transparent)]
    Unspecified(#[from] Unspecified),
}

/// Groups together [`hyper`] and [`hyper_util`] errors, since they
/// cant be converted to one or the other.
#[derive(Debug, thiserror::Error)]
pub enum HyperError {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    HyperUtil(#[from] hyper_util::client::legacy::Error),
}

impl From<hyper::Error> for Error {
    #[inline]
    fn from(value: hyper::Error) -> Self {
        Self::Hyper(HyperError::Hyper(value))
    }
}

impl From<hyper_util::client::legacy::Error> for Error {
    #[inline]
    fn from(value: hyper_util::client::legacy::Error) -> Self {
        Self::Hyper(HyperError::HyperUtil(value))
    }
}

impl From<aws_lc_rs::error::KeyRejected> for Error {
    #[inline]
    fn from(value: aws_lc_rs::error::KeyRejected) -> Self {
        Self::RsaError(RsaError::KeyRejected(value))
    }
}

impl From<aws_lc_rs::error::Unspecified> for Error {
    #[inline]
    fn from(value: aws_lc_rs::error::Unspecified) -> Self {
        Self::RsaError(RsaError::Unspecified(value))
    }
}

#[derive(Debug)]
pub struct ResponseError {
    uri: http::Uri,
    status: StatusCode,
    #[allow(unused)] // mainly around for debug logging
    headers: HeaderMap,
    content: ResponseErrorKind,
    json_error: Option<path_aware_serde::Error<serde_json::Error>>,
}

impl ResponseError {
    pub fn from_parts(uri: http::Uri, parts: http::response::Parts, content: Bytes) -> Self {
        let content = if content.is_empty() {
            ResponseErrorKind::Empty
        } else if content.starts_with(b"{") || content.starts_with(b"[") {
            match serde_json::from_slice(&content) {
                Ok(json) => ResponseErrorKind::Json(json),
                // dont error out if the json is invalid, just fall back to raw text
                // that way we don't lose an error message
                Err(_) => ResponseErrorKind::Text(content),
            }
        } else {
            ResponseErrorKind::Text(content)
        };

        Self {
            uri,
            status: parts.status,
            headers: parts.headers,
            content,
            json_error: None,
        }
    }

    pub fn from_parts_with_json_error(
        uri: http::Uri,
        parts: http::response::Parts,
        content: Bytes,
        error: path_aware_serde::Error<serde_json::Error>,
    ) -> Self {
        let mut new = Self::from_parts(uri, parts, content);
        new.json_error = Some(error);
        new
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn try_extract_json_error_string(json: &serde_json::Value) -> Option<&str> {
            fn get_map_string<'a>(
                map: &'a serde_json::Map<String, serde_json::Value>,
                key: &str,
            ) -> Option<&'a str> {
                match map.get(key)? {
                    serde_json::Value::String(s) => Some(s),
                    _ => None,
                }
            }

            match json {
                serde_json::Value::String(s) => Some(s),
                serde_json::Value::Array(values) => {
                    values.iter().find_map(try_extract_json_error_string)
                }
                serde_json::Value::Object(map) => {
                    // try to get some common error json payload fields first
                    if let Some(message) = get_map_string(map, "message") {
                        return Some(message);
                    }

                    if let Some(message) = get_map_string(map, "error") {
                        return Some(message);
                    }

                    // if none of those exsist, get the value with the longest string,
                    // under the assumption its a human friendly error message
                    map.values()
                        .filter_map(|value| match value {
                            serde_json::Value::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .max_by_key(|string| string.len())
                }
                _ => None,
            }
        }

        // write the common uri/status
        write!(f, "{} - {}", self.uri, self.status)?;

        if let Some(ref json_err) = self.json_error {
            write!(f, " {json_err}")?;
        }

        match self.content {
            ResponseErrorKind::Json(ref json) => match try_extract_json_error_string(json) {
                Some(message) => write!(f, ": {message}"),
                None => Ok(()),
            },
            ResponseErrorKind::Text(ref text) => {
                write!(f, ": {}", bstr::BStr::new(text))
            }
            ResponseErrorKind::Empty => Ok(()),
        }
    }
}

impl std::error::Error for ResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.json_error
            .as_ref()
            .map(|err| err as &(dyn std::error::Error + 'static))
    }
}

#[derive(Debug)]
enum ResponseErrorKind {
    Json(serde_json::Value),
    Text(Bytes),
    Empty,
}
