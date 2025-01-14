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

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Auth(auth) => auth.into_response(),
            Self::Json(_) | Self::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
