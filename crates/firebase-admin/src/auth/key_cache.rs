use std::fmt;
use std::future::Future;
use std::sync::Arc;

use fxhash::{FxBuildHasher, FxHashMap};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use reqwest::header;
use serde::de;
use timestamp::{Duration, Timestamp};
use tokio::sync::{RwLock, RwLockMappedWriteGuard};
use tokio::task::JoinHandle;

use super::{Claims, ValidateTokenError};

const PUBLIC_KEY_URI: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

pub static KEY_CACHE: KeyCache = KeyCache {
    state: RwLock::const_new(CacheState::Empty),
};

pub struct KeyCache {
    state: RwLock<CacheState>,
}

impl fmt::Debug for KeyCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let guard_result = self.state.try_read();

        f.debug_struct("KeyCache")
            .field(
                "state",
                match guard_result.as_deref() {
                    Ok(CacheState::Empty) => &"Empty" as &dyn fmt::Debug,
                    Ok(CacheState::Requesting(_)) => &"Requesting(...)" as &dyn fmt::Debug,
                    Ok(CacheState::Cached(cached)) => cached as &dyn fmt::Debug,
                    Err(_) => &"..." as &dyn fmt::Debug,
                },
            )
            .finish()
    }
}

/// Wrapper around a key id to avoid mixing up with other random strings
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KeyId(Box<str>);

impl KeyId {
    pub fn decode(token: &str) -> crate::Result<Self> {
        let mut header = jsonwebtoken::decode_header(token)?;

        if header.alg != Algorithm::RS256 {
            return Err(ValidateTokenError::UnsupportedAlgo(header.alg).into());
        }

        let kid = header.kid.take().ok_or(ValidateTokenError::MissingKeyId)?;
        Ok(KeyId(kid.into_boxed_str()))
    }
}

#[derive(Clone)]
pub struct Decoder {
    key_id: KeyId,
    decoding_key: Arc<DecodingKey>,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Decoder")
            .field("key_id", &self.key_id)
            .field("decoding_key", &"...")
            .finish()
    }
}

fn get_or_start_requesting_inner<'a>(
    mut guard: tokio::sync::RwLockWriteGuard<'a, CacheState>,
    client: &reqwest::Client,
    force_refresh: bool,
) -> Result<
    RwLockMappedWriteGuard<'a, JoinHandle<crate::Result<CachedKeys>>>,
    RwLockMappedWriteGuard<'a, CachedKeys>,
> {
    macro_rules! map_guard {
        ($guard:expr => $variant:ident) => {{
            tokio::sync::RwLockWriteGuard::map($guard, |state| match state {
                CacheState::$variant(kind) => kind,
                _ => unreachable!(concat!(
                    "wrong kind of CacheState variant encountered, expected ",
                    stringify!($variant)
                )),
            })
        }};
    }

    match &mut *guard {
        CacheState::Empty => (),
        // only start requesting if we either force refresh,
        // or if the current cached key is expired.
        CacheState::Cached(cached) if force_refresh || cached.expires_at < Timestamp::now() => (),
        CacheState::Cached(_) => return Err(map_guard!(guard => Cached)),
        CacheState::Requesting(_) => return Ok(map_guard!(guard => Requesting)),
    }

    // if we made it here, we need to start requesting the keys
    *guard = CacheState::Requesting(tokio::spawn(CachedKeys::get(client)));

    Ok(map_guard!(guard => Requesting))
}

impl KeyCache {
    pub fn try_get_or_start_requesting<'a>(
        &'a self,
        client: &reqwest::Client,
        force_refresh: bool,
    ) -> Option<
        Result<
            RwLockMappedWriteGuard<'a, JoinHandle<crate::Result<CachedKeys>>>,
            RwLockMappedWriteGuard<'a, CachedKeys>,
        >,
    > {
        self.state
            .try_write()
            .ok()
            .map(|guard| get_or_start_requesting_inner(guard, client, force_refresh))
    }

    pub fn get_cached_decoder(&self, key_id: KeyId) -> crate::Result<Result<Decoder, KeyId>> {
        let now = Timestamp::now();

        let Ok(read_guard) = self.state.try_read() else {
            return Ok(Err(key_id));
        };

        match &*read_guard {
            CacheState::Cached(cached) if now < cached.expires_at => {
                let key = cached
                    .keys
                    .get(&key_id)
                    .ok_or(ValidateTokenError::UnknownKeyId)?;
                Ok(Ok(Decoder {
                    key_id,
                    decoding_key: Arc::clone(key),
                }))
            }
            _ => Ok(Err(key_id)),
        }
    }

    pub async fn get_decoder(
        &self,
        key_id: KeyId,
        client: &reqwest::Client,
    ) -> crate::Result<Decoder> {
        let now = Timestamp::now();

        let read_guard = self.state.read().await;

        match &*read_guard {
            CacheState::Cached(cached) if now < cached.expires_at => {
                let key = cached
                    .keys
                    .get(&key_id)
                    .ok_or(ValidateTokenError::UnknownKeyId)?;
                return Ok(Decoder {
                    key_id,
                    decoding_key: Arc::clone(key),
                });
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
                    let key = cached
                        .keys
                        .get(&key_id)
                        .ok_or(ValidateTokenError::UnknownKeyId)?;
                    return Ok(Decoder {
                        key_id,
                        decoding_key: Arc::clone(key),
                    });
                }
            }

            let cached = match std::mem::replace(&mut *write_guard, CacheState::Empty) {
                CacheState::Requesting(handle) => handle.await.unwrap()?,
                CacheState::Cached(mut expired) => {
                    expired.refresh(client).await?;
                    expired
                }
                CacheState::Empty => CachedKeys::get(client).await?,
            };

            // loop back around, just in case google gave us expired keys
            *write_guard = CacheState::Cached(cached);
        }
    }
}

impl Decoder {
    pub fn decode_token(
        &self,
        token: &str,
        validation: &Validation,
    ) -> crate::Result<TokenData<Claims>> {
        jsonwebtoken::decode(token, &self.decoding_key, validation).map_err(crate::Error::from)
    }
}

enum CacheState {
    Empty,
    Requesting(JoinHandle<crate::Result<CachedKeys>>),
    Cached(CachedKeys),
}

pub struct CachedKeys {
    expires_at: Timestamp,
    keys: FxHashMap<KeyId, Arc<DecodingKey>>,
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
        struct DebugKeys<'a>(&'a FxHashMap<KeyId, Arc<DecodingKey>>);

        impl fmt::Debug for DebugKeys<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_set().entries(self.0.keys()).finish()
            }
        }

        f.debug_struct("CachedKeys")
            .field("expires_at", &self.expires_at)
            .field("keys", &DebugKeys(&self.keys))
            .finish()
    }
}

fn get_keys<'a>(
    keys: &'a mut FxHashMap<KeyId, Arc<DecodingKey>>,
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
    keys: &'a mut FxHashMap<KeyId, Arc<DecodingKey>>,
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

            self.keys
                .insert(KeyId(key_id.into()), Arc::new(decoding_key));
        }

        Ok(())
    }
}
