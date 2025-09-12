use std::fmt;

use bytes::BytesMut;
use timestamp::Timestamp;

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Clone, serde::Deserialize)]
pub struct Token<T = ()> {
    #[serde(
        rename = "access_token",
        deserialize_with = "deserialize_access_token_as_header"
    )]
    header: http::HeaderValue,
    #[serde(rename = "expires_in", deserialize_with = "deserialize_expires_at")]
    expires_at: Timestamp,
    /// used indirectly when deserializing tokens from http requests,
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
        let Self {
            header,
            expires_at,
            token_type: Bearer,
        } = self;

        Token {
            header,
            expires_at,
            token_type: (),
        }
    }
}

impl Token {
    #[cfg(feature = "emulator")]
    pub(crate) const EMULATOR_TOKEN: Self = Self {
        expires_at: Timestamp::MAX,
        header: http::HeaderValue::from_static("Bearer: not-real-emulator-token"),
        token_type: (),
    };

    #[cfg(feature = "gcloud")]
    pub(crate) fn new_with_default_expiry_time(
        access_token: &str,
    ) -> Result<Self, http::header::InvalidHeaderValue> {
        // By default most tokens live for an hour, and subtract 10 seconds as a buffer
        const DEFAULT_TOKEN_LIFETIME: timestamp::Duration =
            timestamp::Duration::from_seconds(3600 - 10);

        let header = encode_header(access_token)?;
        Ok(Self {
            header,
            expires_at: Timestamp::now().add_duration(DEFAULT_TOKEN_LIFETIME),
            token_type: (),
        })
    }

    pub fn header(&self) -> &http::HeaderValue {
        &self.header
    }

    pub fn access_token(&self) -> &str {
        std::str::from_utf8(&self.header.as_bytes()[BEARER_PREFIX.len()..])
            .expect("this should be valid")
    }

    pub fn expires_at(&self) -> timestamp::Timestamp {
        self.expires_at
    }

    pub fn valid_for(&self) -> Result<std::time::Duration, timestamp::Timestamp> {
        let now = timestamp::Timestamp::now();

        if now <= self.expires_at {
            Ok((self.expires_at - now).into())
        } else {
            Err(now)
        }
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

fn deserialize_access_token_as_header<'de, D>(
    deserializer: D,
) -> Result<http::HeaderValue, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'vde> serde::de::Visitor<'vde> for Visitor {
        type Value = http::HeaderValue;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an access token")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match std::str::from_utf8(v) {
                Ok(s) => self.visit_str(s),
                Err(err) => Err(E::invalid_value(
                    serde::de::Unexpected::Bytes(v),
                    &err.to_string().as_str(),
                )),
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.is_empty() {
                return Err(E::invalid_length(0, &self));
            }

            encode_header(v).map_err(|err| {
                E::invalid_value(serde::de::Unexpected::Str(v), &err.to_string().as_str())
            })
        }
    }

    deserializer.deserialize_str(Visitor)
}

fn encode_header(
    access_token: &str,
) -> Result<http::HeaderValue, http::header::InvalidHeaderValue> {
    let len = BEARER_PREFIX.len() + access_token.len();

    let mut dst = BytesMut::with_capacity(len);
    dst.extend_from_slice(BEARER_PREFIX.as_bytes());
    dst.extend_from_slice(access_token.as_bytes());

    let buf = dst.freeze();

    http::HeaderValue::from_maybe_shared(buf)
}
