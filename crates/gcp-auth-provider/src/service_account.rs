use std::borrow::Cow;
use std::cell::RefCell;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, io};

use aws_lc_rs::error::KeyRejected;
use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::rsa::KeyPair;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bytes::Bytes;
use http::HeaderValue;
use rustls_pki_types::pem::PemObject;
use timestamp::Timestamp;

use crate::{Error, ProjectId, Scopes};

const FORM_URL_ENCODED: HeaderValue = HeaderValue::from_static("application/x-www-form-urlencoded");

pub struct ServiceAccount {
    client: crate::client::HttpsClient,
    signer: Signer,
    token_uri: http::Uri,
    client_email: Box<str>,
    private_key_id: Box<str>,
    subject: Option<Box<str>>,
    audience: Option<Box<str>>,
}

impl fmt::Debug for ServiceAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceAccount")
            .field("client_email", &self.client_email)
            .field("token_uri", &self.token_uri)
            .field("subject", &self.subject)
            .field("audience", &self.audience)
            .finish_non_exhaustive()
    }
}

impl ServiceAccount {
    pub fn new_from_json_bytes(bytes: &[u8]) -> Result<(Self, ProjectId), Error> {
        ServiceAccountKey::from_json_bytes(bytes)
            .map_err(Error::Json)
            .and_then(Self::new_from_key)
    }

    pub async fn new_from_env() -> Result<Option<(Self, ProjectId)>, Error> {
        let Some(path) = std::env::var_os("GOOGLE_APPLICATION_CREDENTIALS") else {
            return Ok(None);
        };

        if path.is_empty() {
            return Ok(None);
        }

        Self::new_from_json_file(Path::new(path.as_os_str()))
            .await
            .map(Some)
    }

    pub async fn new_from_json_file(path: &Path) -> Result<(Self, ProjectId), Error> {
        ServiceAccountKey::from_path(path, Self::new_from_key).await?
    }

    pub(super) async fn try_load(
        ctx: &mut crate::InitContext,
    ) -> crate::Result<Option<(Self, ProjectId)>> {
        ServiceAccountKey::from_env(move |key| {
            let client = match ctx.https.take() {
                Some(client) => client,
                None => crate::client::HttpsClient::new_https()?,
            };

            match Self::from_parts(client, key) {
                Ok(parts) => Ok(parts),
                Err((error, client)) => {
                    ctx.https = Some(client);
                    Err(error)
                }
            }
        })
        .await?
        .transpose()
    }

    pub fn new_from_key(key: ServiceAccountKey<'_>) -> Result<(Self, ProjectId), Error> {
        let client = crate::client::HttpsClient::new_https()?;
        Self::from_parts(client, key).map_err(|(error, _)| error)
    }

    fn from_parts(
        client: crate::client::HttpsClient,
        key: ServiceAccountKey<'_>,
    ) -> Result<(Self, ProjectId), (Error, crate::client::HttpsClient)> {
        let private_key = match key.private_key {
            Ok(key) => key,
            Err(err) => return Err((err.into(), client)),
        };

        let project_id = ProjectId::new_cow(key.project_id);
        Ok((
            ServiceAccount {
                client,
                subject: None,
                audience: None,
                private_key_id: Box::from(key.private_key_id),
                client_email: Box::from(key.client_email),
                token_uri: key.token_uri,
                signer: Signer {
                    key: private_key,
                    rng: SystemRandom::new(),
                },
            },
            project_id,
        ))
    }

    fn encode_body(&self, scopes: Scopes) -> Result<Bytes, Error> {
        const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
        static MAX_URL_ENCODED_LEN: AtomicUsize = AtomicUsize::new(0);

        let jwt = Claims::new(self, scopes).encode_jwt(self)?;

        let cap = MAX_URL_ENCODED_LEN.load(Ordering::Relaxed);
        let dst = String::with_capacity(if cap == 0 {
            GRANT_TYPE.len() + jwt.len() + 32
        } else {
            cap
        });

        let body = form_urlencoded::Serializer::new(dst)
            .extend_pairs(&[("grant_type", GRANT_TYPE), ("assertion", jwt.as_str())])
            .finish()
            .into_bytes();

        MAX_URL_ENCODED_LEN.fetch_max(body.len(), Ordering::Relaxed);
        Ok(Bytes::from(body))
    }

    fn encode_request(
        &self,
        scopes: Scopes,
    ) -> Result<http::Request<crate::client::BytesBody>, Error> {
        let body = self.encode_body(scopes)?;

        http::Request::builder()
            .uri(self.token_uri.clone())
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, FORM_URL_ENCODED)
            .body(crate::client::BytesBody::new(body))
            .map_err(|err| Error::Io(io::Error::new(io::ErrorKind::InvalidData, err)))
    }
}
impl crate::BaseTokenProvider for ServiceAccount {
    #[inline]
    fn name(&self) -> &'static str {
        "service account"
    }
}

impl crate::ScopedTokenProvider for ServiceAccount {
    #[inline]
    fn get_scoped_token(&self, scopes: crate::Scopes) -> crate::GetTokenFuture<'_> {
        match self.encode_request(scopes) {
            Ok(request) => crate::GetTokenFuture::new_https(&self.client, request),
            Err(error) => crate::GetTokenFuture::new_error(error),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(bound = "'de: 'a")]
pub struct ServiceAccountKey<'a> {
    #[serde(with = "serde_helpers::borrow")]
    project_id: Cow<'a, str>,
    #[serde(with = "serde_helpers::borrow")]
    private_key_id: Cow<'a, str>,
    #[serde(deserialize_with = "deserialize_private_key")]
    private_key: Result<KeyPair, KeyRejected>,
    #[serde(with = "serde_helpers::borrow")]
    client_email: Cow<'a, str>,
    #[serde(with = "serde_helpers::from_str")]
    token_uri: http::Uri,
}

fn deserialize_private_key<'de, D>(
    deserializer: D,
) -> Result<Result<KeyPair, KeyRejected>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_bytes(KeyPairVisitor)
}

struct KeyPairVisitor;

impl<'de> serde::de::Visitor<'de> for KeyPairVisitor {
    type Value = Result<KeyPair, KeyRejected>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an encoded private key")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v.as_bytes())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let pem = rustls_pki_types::PrivateKeyDer::pem_slice_iter(v)
            .next()
            .ok_or_else(|| E::custom("no private key found"))?
            .map_err(|err| E::custom(format!("invalid private key: {err}")))?;

        Ok(KeyPair::from_pkcs8(pem.secret_der()))
    }
}

#[derive(Debug)]
struct Signer {
    key: KeyPair,
    rng: SystemRandom,
}

impl Signer {
    fn sign(&self, input: &[u8], dst: &mut Vec<u8>) -> Result<(), Error> {
        dst.resize(self.key.public_modulus_len(), 0);

        self.key.sign(
            &aws_lc_rs::signature::RSA_PKCS1_SHA256,
            &self.rng,
            input,
            dst,
        )?;

        Ok(())
    }
}

impl<'a> ServiceAccountKey<'a> {
    pub fn from_json_bytes(
        bytes: &'a [u8],
    ) -> Result<Self, path_aware_serde::Error<serde_json::Error>> {
        path_aware_serde::json::deserialize_slice(bytes)
    }

    pub async fn from_path<F, O>(path: &Path, visitor: F) -> Result<O, Error>
    where
        for<'b> F: FnOnce(ServiceAccountKey<'b>) -> O,
    {
        let bytes = tokio::fs::read(path).await?;
        let key = ServiceAccountKey::from_json_bytes(&bytes)?;
        Ok(visitor(key))
    }

    pub async fn from_env<F, O>(visitor: F) -> Result<Option<O>, Error>
    where
        for<'b> F: FnOnce(ServiceAccountKey<'b>) -> O,
    {
        let Some(path) = std::env::var_os("GOOGLE_APPLICATION_CREDENTIALS") else {
            return Ok(None);
        };

        if path.is_empty() {
            return Ok(None);
        }

        Self::from_path(Path::new(&path), visitor).await.map(Some)
    }
}

#[derive(serde::Serialize)]
struct Claims<'a> {
    iss: &'a str,
    aud: Audience<'a>,
    exp: i64,
    iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<&'a str>,
    #[serde(serialize_with = "crate::scope::serialize_scope_urls")]
    scope: Scopes,
}

enum Audience<'a> {
    Str(&'a str),
    Uri(&'a http::Uri),
}

impl serde::Serialize for Audience<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Str(s) => serializer.serialize_str(s),
            Self::Uri(uri) => serializer.collect_str(uri),
        }
    }
}

impl<'a> Claims<'a> {
    fn new(service_acct: &'a ServiceAccount, scope: Scopes) -> Self {
        let now = Timestamp::now();

        Self {
            iss: &service_acct.client_email,
            aud: match service_acct.audience.as_deref() {
                Some(aud) => Audience::Str(aud),
                None => Audience::Uri(&service_acct.token_uri),
            },
            iat: now.as_seconds(),
            exp: now
                .add_duration(timestamp::Duration::from_seconds(3600 - 5))
                .as_seconds(),
            sub: Some(
                service_acct
                    .subject
                    .as_deref()
                    .unwrap_or(&service_acct.client_email),
            ),
            scope,
        }
    }

    fn encode_jwt(&self, svc: &ServiceAccount) -> crate::Result<String> {
        static MAX_JWT_CAPACITY: AtomicUsize = AtomicUsize::new(0);

        thread_local! {
            static ENCODE_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024));
        }

        let cap = MAX_JWT_CAPACITY.load(Ordering::Relaxed);
        let mut jwt = String::with_capacity(if cap == 0 { 512 } else { cap });

        ENCODE_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            buf.clear();

            let header = Header {
                alg: "RS256",
                typ: "JWT",
                kid: &svc.private_key_id,
            };

            serde_json::to_writer(&mut *buf, &header).map_err(|err| Error::Json(err.into()))?;

            URL_SAFE_NO_PAD.encode_string(&*buf, &mut jwt);
            jwt.push('.');

            buf.clear();
            self.encode_claims(&mut *buf)?;

            URL_SAFE_NO_PAD.encode_string(&*buf, &mut jwt);

            svc.signer.sign(jwt.as_bytes(), &mut *buf)?;
            jwt.push('.');
            URL_SAFE_NO_PAD.encode_string(&*buf, &mut jwt);

            Ok(()) as Result<(), Error>
        })?;

        MAX_JWT_CAPACITY.fetch_max(jwt.len(), Ordering::Relaxed);

        Ok(jwt)
    }

    fn encode_claims(&self, buf: &mut Vec<u8>) -> crate::Result<()> {
        use serde::Serialize;
        self.serialize(&mut serde_json::Serializer::new(buf))
            .map_err(|err| Error::Json(err.into()))
    }
}

#[derive(serde::Serialize)]
struct Header<'a> {
    alg: &'a str,
    typ: &'a str,
    kid: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScopedTokenProvider;

    #[tokio::test]
    async fn test_service_account() -> Result<(), Error> {
        let (service_acct, project_id) = ServiceAccount::try_load(&mut Default::default())
            .await?
            .unwrap();

        println!("{project_id:?}");
        println!("{service_acct:#?}");

        let token = service_acct.get_scoped_token(Scopes::GCS_READ_ONLY).await?;

        println!("{token:#?}");

        Ok(())
    }
}
