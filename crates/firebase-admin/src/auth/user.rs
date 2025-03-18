use std::sync::Arc;

use timestamp::Timestamp;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    created_at: Timestamp,
    #[serde(default)]
    disabled: bool,
    email: Arc<str>,
    #[serde(alias = "localId")]
    uid: Arc<str>,
    valid_since: Option<Timestamp>,
    photo_url: Option<Box<str>>,
    last_login_at: Option<Timestamp>,
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
