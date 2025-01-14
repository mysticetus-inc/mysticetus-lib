mod error;
mod key_cache;
mod layer_service;
use std::collections::HashSet;

pub use error::AuthError;
use jsonwebtoken::Validation;
use key_cache::KeyCache;
pub use layer_service::ValidateIdTokenLayer;
use timestamp::Timestamp;

#[derive(Debug)]
struct ValidateIdTokenShared {
    project_id: &'static str,
    validation: Validation,
    key_cache: KeyCache,
}

impl ValidateIdTokenShared {
    fn new(project_id: &'static str, client: reqwest::Client, start_requesting_keys: bool) -> Self {
        Self {
            project_id,
            validation: make_validation(project_id),
            key_cache: KeyCache::new(client, start_requesting_keys),
        }
    }
}

fn make_validation(project_id: &'static str) -> Validation {
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

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Claims {
    aud: Box<str>,
    auth_time: Timestamp,
    email: Box<str>,
    email_verified: bool,
    exp: Timestamp,
    iat: Timestamp,
    name: Option<Box<str>>,
    user_id: Box<str>,
}
