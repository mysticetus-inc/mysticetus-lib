use http::StatusCode;
use jsonwebtoken::Algorithm;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("jwt has no key id")]
    MissingKeyId,
    #[error("jwt key id is unknown")]
    UnknownKeyId,
    #[error("unsupported jwt algorithm. expected RS256, got {0:?}")]
    UnsupportedAlgo(Algorithm),
    #[error("no Bearer token given")]
    NoBearerToken,
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Bearer token had invalid ascii characters in it: {0}")]
    InvalidToken(#[from] http::header::ToStrError),
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        // for token/auth errors we don't want to return detailed error info
        StatusCode::FORBIDDEN.into_response()
    }
}
