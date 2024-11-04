use std::fmt;

use crate::coords::Size;
use crate::feature::DrawError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    InvalidSize(#[from] InvalidSize),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Gcs(#[from] small_gcs::Error),
    #[error("mapbox tileserver did not return a 'Content-Type' header")]
    MissingContentType,
    #[error(transparent)]
    UnknownContentType(#[from] mime_guess::mime::FromStrError),
    #[error("unsupported tile content/mime type: {0}")]
    UnsupportedContentType(mime_guess::mime::Mime),
    #[error(transparent)]
    DrawError(#[from] DrawError),
    #[error(transparent)]
    PngDecodeError(#[from] png::DecodingError),
    #[error(transparent)]
    PngEncodeError(#[from] png::EncodingError),
    #[error(transparent)]
    TaskError(#[from] tokio::task::JoinError),
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        let (kind, status) = match &self {
            Self::InvalidSize(_) => ("invalid_size", StatusCode::BAD_REQUEST),
            _ => ("internal", StatusCode::INTERNAL_SERVER_ERROR),
        };

        error!(message = "error generating map", error = ?self);

        match serde_error::encode(kind, self) {
            Ok(body) => (status, body).into_response(),
            Err(resp) => resp,
        }
    }
}
#[cfg(feature = "axum")]
mod serde_error {
    use std::fmt;

    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    pub(super) fn encode<E: fmt::Display>(
        kind: &str,
        error: E,
    ) -> Result<Vec<u8>, axum::response::Response> {
        match serde_json::to_vec(&SerializeErr { kind, error }) {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
        }
    }

    struct SerializeErr<'a, E> {
        kind: &'a str,
        error: E,
    }

    impl<E: fmt::Display> serde::Serialize for SerializeErr<'_, E> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeMap;

            let mut map = serializer.serialize_map(Some(2))?;

            map.serialize_entry("kind", self.kind)?;
            map.serialize_entry("error", &SerializeDisplay(&self.error))?;

            map.end()
        }
    }

    struct SerializeDisplay<'a, E>(&'a E);

    impl<E: fmt::Display> serde::Serialize for SerializeDisplay<'_, E> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_str(self.0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSize {
    size: Size<u32>,
}
impl std::error::Error for InvalidSize {}

impl From<Size<u32>> for InvalidSize {
    fn from(size: Size<u32>) -> Self {
        Self { size }
    }
}

impl InvalidSize {
    pub fn new(size: Size<u32>) -> Self {
        Self { size }
    }

    pub fn get(&self) -> Size<u32> {
        self.size
    }
}

impl fmt::Display for InvalidSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MAX_SIZE: u32 = (i32::MAX / 4) as u32;

        if self.size.height == 0 && self.size.width == 0 {
            f.write_str("image must be at least 1x1 pixels")
        } else if self.size.height == 0 {
            f.write_str("image height must be at least 1 pixel")
        } else if self.size.width == 0 {
            f.write_str("image width must be at least 1 pixel")
        } else if self.size.height >= MAX_SIZE {
            write!(
                f,
                "maximum image height exceeded: {} (max is {MAX_SIZE})",
                self.size.height
            )
        } else if self.size.width >= MAX_SIZE {
            write!(
                f,
                "maximum image width exceeded: {} (max is {MAX_SIZE})",
                self.size.height
            )
        } else {
            write!(
                f,
                "invalid image size: {}x{}",
                self.size.width, self.size.height
            )
        }
    }
}
