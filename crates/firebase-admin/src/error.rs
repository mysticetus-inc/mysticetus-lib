use std::borrow::Cow;

use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] path_aware_serde::Error<serde_json::Error>),
    #[error(transparent)]
    Auth(#[from] crate::auth::AuthError),
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::Auth(value.into())
    }
}

impl Error {
    pub fn to_response_parts(&self) -> (StatusCode, Cow<'static, str>) {
        match self {
            Self::Auth(auth) => auth.to_response_parts(),
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