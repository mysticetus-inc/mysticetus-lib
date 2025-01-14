use std::fmt;

use timestamp::{Duration, Timestamp};

// By default most tokens live for an hour.
const DEFAULT_TOKEN_LIFETIME: Duration = Duration::from_seconds(3600);

#[derive(serde::Deserialize)]
pub struct Token {
    access_token: Box<str>,
    #[serde(rename = "expires_at", deserialize_with = "deserialize_expires_at")]
    expires_at: Timestamp,
}

impl Token {
    pub(crate) fn new_with_default_expiry_time(access_token: Box<str>) -> Self {
        Self {
            access_token,
            expires_at: Timestamp::now().add_duration(DEFAULT_TOKEN_LIFETIME),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("access_token", &"...")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

fn deserialize_expires_at<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let expires_in: i64 = serde::Deserialize::deserialize(deserializer)?;
    Ok(Timestamp::now().add_seconds(expires_in))
}
