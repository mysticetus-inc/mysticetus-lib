use std::collections::BTreeMap;
use std::fmt;
use std::future::Future;
use std::sync::Arc;

use fxhash::{FxBuildHasher, FxHashMap};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::header;
use serde::de;
use timestamp::{Duration, Timestamp};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use super::{AuthError, Claims};

const PUBLIC_KEY_URI: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

pub struct KeyCache {
    // TODO: replace with a short lived http client
    // that only lives long enough to get the keys,
    // then disconnects (since we'll be caching for
    // several hours)
    client: reqwest::Client,
    state: RwLock<CacheState>,
}

impl fmt::Debug for KeyCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let guard_result = self.state.try_read();

        f.debug_struct("KeyCache")
            .field("client", &self.client)
            .field("state", match guard_result.as_deref() {
                Ok(CacheState::Empty) => &"Empty" as &dyn fmt::Debug,
                Ok(CacheState::Requesting(_)) => &"Requesting" as &dyn fmt::Debug,
                Ok(CacheState::Cached(cached)) => cached as &dyn fmt::Debug,
                Err(_) => &"..." as &dyn fmt::Debug,
            })
            .finish()
    }
}

impl KeyCache {
    pub fn new(client: reqwest::Client, start_requesting: bool) -> Self {
        Self {
            state: RwLock::new(if start_requesting {
                let client = client.clone();
                CacheState::Requesting(tokio::spawn(CachedKeys::get(&client)))
            } else {
                CacheState::Empty
            }),
            client,
        }
    }

    async fn get_decoding_key(&self, kid: &str) -> crate::Result<Arc<DecodingKey>> {
        let now = Timestamp::now();

        let read_guard = self.state.read().await;

        match &*read_guard {
            CacheState::Cached(cached) if now < cached.expires_at => {
                let key = cached.keys.get(kid).ok_or(AuthError::UnknownKeyId)?;
                return Ok(Arc::clone(key));
            }
            // otherwise we need to get a write guard and handle refreshing/a pending request
            _ => (),
        }

        drop(read_guard);

        let mut write_guard = self.state.write().await;

        loop {
            if let CacheState::Cached(ref mut cached) = *write_guard {
                // waiting for a write guard and/or refreshing may take time,
                // so use a new 'now' timestamp when we check.
                if Timestamp::now() < cached.expires_at {
                    let key = cached.keys.get(kid).ok_or(AuthError::UnknownKeyId)?;
                    return Ok(Arc::clone(key));
                }
            }

            let cached = match std::mem::replace(&mut *write_guard, CacheState::Empty) {
                CacheState::Requesting(handle) => handle.await.unwrap()?,
                CacheState::Cached(mut expired) => {
                    expired.refresh(&self.client).await?;
                    expired
                }
                CacheState::Empty => CachedKeys::get(&self.client).await?,
            };

            // loop back around, just in case google gave us expired keys
            *write_guard = CacheState::Cached(cached);
        }
    }

    pub async fn validate_token(
        &self,
        token: &str,
        validation: &Validation,
    ) -> crate::Result<TokenData<Claims>> {
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.as_deref().ok_or(AuthError::MissingKeyId)?;

        let decoding_key = self.get_decoding_key(kid).await?;

        jsonwebtoken::decode(token, &decoding_key, validation).map_err(crate::Error::from)
    }
}

enum CacheState {
    Empty,
    Requesting(JoinHandle<crate::Result<CachedKeys>>),
    Cached(CachedKeys),
}

pub struct CachedKeys {
    expires_at: Timestamp,
    keys: FxHashMap<Box<str>, Arc<DecodingKey>>,
}

impl CachedKeys {
    fn get(client: &reqwest::Client) -> impl Future<Output = crate::Result<Self>> + 'static {
        let client = client.clone();
        async move {
            let mut keys = FxHashMap::with_capacity_and_hasher(8, FxBuildHasher::default());
            let expires_at = get_keys(&mut keys, &client).await?;
            Ok(Self { expires_at, keys }) as crate::Result<_>
        }
    }

    async fn refresh(&mut self, client: &reqwest::Client) -> crate::Result<()> {
        self.expires_at = get_keys(&mut self.keys, client).await?;
        Ok(())
    }
}

impl fmt::Debug for CachedKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachedKeys")
            .field("expires_at", &self.expires_at)
            .field("keys", &self.keys.keys())
            .finish()
    }
}

fn get_keys<'a>(
    keys: &'a mut FxHashMap<Box<str>, Arc<DecodingKey>>,
    client: &reqwest::Client,
) -> impl Future<Output = crate::Result<Timestamp>> + 'a {
    fn parse_max_age(header: &header::HeaderValue) -> Option<Duration> {
        let str = header.to_str().ok()?;

        let max_age_str = str
            .split_terminator(',')
            .map(str::trim)
            .find_map(|s| s.strip_prefix("max-age="))?;

        let max_age_secs = max_age_str.parse::<i64>().ok()?;

        Some(Duration::from_seconds(max_age_secs))
    }

    let (client, request_result) = client.get(PUBLIC_KEY_URI).build_split();

    async move {
        let request = request_result?;
        let resp = client.execute(request).await?;

        let max_age = resp
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(parse_max_age)
            .unwrap_or(Duration::from_minutes(60));

        let expires_at = Timestamp::now() + max_age;

        let body = resp.bytes().await?;

        path_aware_serde::json::deserialize_slice_seed(KeysVisitor { keys }, &body)?;

        Ok(expires_at)
    }
}

struct KeysVisitor<'a> {
    keys: &'a mut FxHashMap<Box<str>, Arc<DecodingKey>>,
}

impl<'de> de::DeserializeSeed<'de> for KeysVisitor<'_> {
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> de::Visitor<'de> for KeysVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a json map of kid -> pem key")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        use std::borrow::Cow;
        self.keys.clear();

        while let Some((key_id, key)) = map.next_entry::<Cow<'de, str>, Cow<'de, str>>()? {
            let decoding_key =
                DecodingKey::from_rsa_pem(key.as_bytes()).map_err(de::Error::custom)?;

            self.keys.insert(key_id.into(), Arc::new(decoding_key));
        }

        Ok(())
    }
}
