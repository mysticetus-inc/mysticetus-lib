use std::fmt;

use hyper::body::Incoming;
use timestamp::{Duration, Timestamp};

use crate::Error;

// By default most tokens live for an hour, and subtract 10 seconds as a buffer
const DEFAULT_TOKEN_LIFETIME: Duration = Duration::from_seconds(3600 - 10);

#[derive(serde::Deserialize)]
pub struct Token<T = ()> {
    access_token: Box<str>,
    #[serde(rename = "expires_in", deserialize_with = "deserialize_expires_at")]
    expires_at: Timestamp,
    /// used when deserializing tokens from the metadata server,
    /// to ensure that `"token_type": "Bearer"`
    token_type: T,
}

#[derive(Debug)]
pub(crate) struct Bearer;

impl<'de> serde::Deserialize<'de> for Bearer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'vde> serde::de::Visitor<'vde> for Visitor {
            type Value = Bearer;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a `Bearer` token_type")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "Bearer" {
                    Ok(Bearer)
                } else {
                    Err(E::invalid_value(serde::de::Unexpected::Str(v), &self))
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Token<Bearer> {
    #[inline]
    pub(crate) fn into_unit_token_type(self) -> Token {
        Token {
            access_token: self.access_token,
            expires_at: self.expires_at,
            token_type: (),
        }
    }
}

impl<T: serde::de::DeserializeOwned> Token<T> {
    pub(crate) async fn deserialize_from_response(
        uri: &http::Uri,
        resp: http::Response<Incoming>,
    ) -> Result<Self, Error> {
        if resp.status().is_success() {
            Self::deserialize_from_body(uri, resp.into_body()).await
        } else {
            Err(
                crate::error::ResponseError::from_response(uri.clone(), resp)
                    .await?
                    .into(),
            )
        }
    }

    pub(crate) async fn deserialize_from_body(
        uri: &http::Uri,
        body: Incoming,
    ) -> Result<Self, Error> {
        let bytes = crate::util::collect_body(body).await?;

        if bytes.is_empty() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("empty body recieved from '{uri:?}'"),
            )));
        }

        path_aware_serde::json::deserialize_slice(&bytes).map_err(Error::Json)
    }
}

impl Token {
    pub(crate) fn new_with_default_expiry_time(access_token: Box<str>) -> Self {
        Self {
            access_token,
            expires_at: Timestamp::now().add_duration(DEFAULT_TOKEN_LIFETIME),
            token_type: (),
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn expires_at(&self) -> timestamp::Timestamp {
        self.expires_at
    }
}

impl<T> fmt::Debug for Token<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("access_token", &"...") // dont log tokens
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
