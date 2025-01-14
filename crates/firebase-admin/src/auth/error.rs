use std::borrow::Cow;

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
    #[error("not a Bearer token")]
    NotABearerToken,
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Bearer token had invalid ascii characters in it: {0}")]
    InvalidToken(#[from] http::header::ToStrError),
}

impl AuthError {
    pub fn to_response_parts(&self) -> (StatusCode, Cow<'static, str>) {
        // don't give any detail on why a token failed to validate, only
        // if a token is missing or is obviusly the wrong kind.
        let message = match self {
            Self::NoBearerToken => "missing Authorization Bearer token",
            Self::NotABearerToken => "Authorization header not a Bearer token",
            _ => "invalid Bearer token",
        };

        (StatusCode::UNAUTHORIZED, Cow::Borrowed(message))
    }
}
