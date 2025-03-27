use std::collections::HashSet;
use std::sync::OnceLock;

use http::{HeaderMap, HeaderValue, header};
use jsonwebtoken::Validation;

use super::ValidateTokenError;
use crate::auth::key_cache::{KEY_CACHE, KeyId};

pub async fn authorize_request(
    project_id: &'static str,
    headers: &mut HeaderMap,
    make_client: impl FnOnce() -> reqwest::Client,
) -> crate::Result<super::Token> {
    let token_header = extract_header(headers)?;

    let validation = get_or_create_validation(project_id);

    let result = authorize_from_header(&token_header, &*validation, make_client).await;
    // re-insert the header, in-case some other middleware needs it for something.
    headers.insert(header::AUTHORIZATION, token_header);

    result
}

pub(super) async fn authorize_from_header(
    token_header: &HeaderValue,
    validation: &Validation,
    make_client: impl FnOnce() -> reqwest::Client,
) -> crate::Result<super::Token> {
    let (token_str, key_id) = match try_authorize_from_header(token_header, validation)? {
        Ok(decoded) => return Ok(decoded),
        Err((token_str, key_id)) => (token_str, key_id),
    };

    let decoder = KEY_CACHE.get_decoder(key_id, &make_client()).await?;

    decoder.decode_token(token_str, validation)
}

/// attempts to authorize from a cached decoder, returning Ok(Err((token_str, key_id)))
/// if we need to make a request to get a decoder.
pub(super) fn try_authorize_from_header<'a>(
    token_header: &'a HeaderValue,
    validation: &Validation,
) -> crate::Result<Result<super::Token, (&'a str, KeyId)>> {
    let token_str = token_header
        .to_str()
        .map_err(ValidateTokenError::InvalidToken)?
        .trim_start_matches("Bearer ");

    let key_id = KeyId::decode(token_str)?;

    match KEY_CACHE.get_cached_decoder(key_id)? {
        Ok(decoder) => decoder.decode_token(token_str, &validation).map(Ok),
        Err(key_id) => Ok(Err((token_str, key_id))),
    }
}

pub(super) fn extract_header(
    headers: &mut http::HeaderMap,
) -> Result<HeaderValue, ValidateTokenError> {
    let header = headers
        .remove(header::AUTHORIZATION)
        .ok_or(ValidateTokenError::NoBearerToken)?;

    if header.as_bytes().starts_with(b"Bearer ") {
        Ok(header)
    } else {
        headers.insert(header::AUTHORIZATION, header);
        Err(ValidateTokenError::NotABearerToken)
    }
}

pub fn get_or_create_validation(project_id: &'static str) -> std::borrow::Cow<'static, Validation> {
    static VALIDATION: OnceLock<(&'static str, Validation)> = OnceLock::new();

    let (cached_project_id, cached_validation) =
        VALIDATION.get_or_init(move || (project_id, create_validation(project_id)));

    if *cached_project_id == project_id {
        std::borrow::Cow::Borrowed(cached_validation)
    } else {
        std::borrow::Cow::Owned(create_validation(project_id))
    }
}

fn create_validation(project_id: &'static str) -> Validation {
    #[inline]
    fn hash_set_from(value: impl Into<String>) -> HashSet<String> {
        let mut set = HashSet::with_capacity(1);
        set.insert(value.into());
        set
    }

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.required_spec_claims.reserve(4);
    validation.required_spec_claims.extend([
        "iat".to_owned(),
        "aud".to_owned(),
        "iss".to_owned(),
        "sub".to_owned(),
    ]);
    validation.aud = Some(hash_set_from(project_id));
    validation.validate_aud = true;

    validation.iss = Some(hash_set_from(format!(
        "https://securetoken.google.com/{project_id}"
    )));

    validation
}
