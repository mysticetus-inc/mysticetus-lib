use std::borrow::Cow;

use jsonwebtoken::Validation;
use timestamp::Timestamp;

mod error;
pub use error::ValidateTokenError;

mod key_cache;
use key_cache::KEY_CACHE;

mod layer_service;
pub use layer_service::{ValidateIdTokenLayer, ValidateIdTokenService};

pub mod manager;
pub use manager::AuthManager;

pub mod validate;

pub mod user;
pub use user::UserInfo;

pub type Token = jsonwebtoken::TokenData<Claims>;

#[derive(Debug)]
struct ValidateIdTokenShared {
    validation: Cow<'static, Validation>,
    client: reqwest::Client,
}

impl ValidateIdTokenShared {
    fn new(project_id: &'static str, client: reqwest::Client, start_requesting_keys: bool) -> Self {
        if start_requesting_keys {
            KEY_CACHE.try_get_or_start_requesting(&client, false);
        }

        Self {
            validation: validate::get_or_create_validation(project_id),
            client,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Claims {
    pub aud: Box<str>,
    pub auth_time: Timestamp,
    pub email: Box<str>,
    pub email_verified: bool,
    pub exp: Timestamp,
    pub iat: Timestamp,
    pub name: Option<Box<str>>,
    pub user_id: Box<str>,
}
