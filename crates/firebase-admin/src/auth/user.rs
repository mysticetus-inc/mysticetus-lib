use std::sync::Arc;

use timestamp::Timestamp;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(deserialize_with = "Timestamp::deserialize_from_millis")]
    created_at: Timestamp,
    #[serde(default)]
    disabled: bool,
    email: Arc<str>,
    #[serde(alias = "localId")]
    uid: Arc<str>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "Timestamp::deserialize_from_millis_opt"
    )]
    valid_since: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    photo_url: Option<Box<str>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "Timestamp::deserialize_from_millis_opt"
    )]
    last_login_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<Box<str>>,
}

impl UserInfo {
    pub fn uid(&self) -> &Arc<str> {
        &self.uid
    }

    pub fn email(&self) -> &Arc<str> {
        &self.email
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn photo_url(&self) -> Option<&str> {
        self.photo_url.as_deref()
    }

    pub fn last_login_at(&self) -> Option<Timestamp> {
        self.last_login_at
    }
}
