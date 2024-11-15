use std::fmt;

use shared::Shared;

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    pub access_token: Shared<str>,
    pub username: Shared<str>,
    pub style_id: Shared<str>,
}

// manual impl, since we dont want debug printing leaking the access token,
// even though its technically not a secure value
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("access_token", &"...")
            .field("username", &self.username.as_ref())
            .field("style_id", &self.style_id.as_ref())
            .finish()
    }
}
