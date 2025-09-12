use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use std::{fmt, io};

use aws_lc_rs::error::KeyRejected;
use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::rsa::KeyPair;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bytes::Bytes;
use http::HeaderValue;
use rustls_pki_types::pem::PemObject;
use timestamp::Timestamp;

use super::InitContext;
use crate::client::HttpsClient;
use crate::providers::LoadProviderResult;
use crate::util::{CowMut, ReadFuture};
use crate::{Error, ProjectId, Scopes};

const FORM_URL_ENCODED: HeaderValue = HeaderValue::from_static("application/x-www-form-urlencoded");

const DEFAULT_ENV_NAME: &str = "GOOGLE_APPLICATION_CREDENTIALS";

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

    pub fn new_from_env() -> Option<TryLoadFuture<'static>> {
        TryLoadFuture::new(CowMut::Owned(None))
    }

    pub fn new_from_path(path: impl Into<PathBuf>) -> TryLoadFuture<'static> {
        TryLoadFuture::new_from_path(CowMut::Owned(None), path.into())
    }

    pub(super) fn try_load(ctx: &mut InitContext) -> Option<TryLoadFuture<'_>> {
        TryLoadFuture::new(CowMut::RefMut(&mut ctx.https))
    }

    pub fn new_from_key(key: ServiceAccountKey<'_>) -> Result<(Self, ProjectId), Error> {
        Self::from_parts(&mut None, key).map_err(|(error, _)| error)
    }

    pub(crate) fn from_parts(
        client: &mut Option<crate::client::HttpsClient>,
        key: ServiceAccountKey<'_>,
    ) -> Result<(Self, ProjectId), (Error, Option<crate::client::HttpsClient>)> {
        let private_key = match key.private_key {
            Ok(key) => key,
            Err(err) => return Err((err.into(), client.take())),
        };

        let client = match client.take() {
            Some(client) => client,
            None => match crate::client::HttpsClient::new_https() {
                Ok(client) => client,
                Err(error) => return Err((error.into(), None)),
            },
        };

        let project_id = ProjectId::from(key.project_id);
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
        let body_str = encode_jwt_body(self, scopes)?;
        Ok(Bytes::from(body_str.into_bytes()))
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
impl super::BaseTokenProvider for ServiceAccount {
    #[inline]
    fn name(&self) -> &'static str {
        "service account"
    }
}

impl super::ScopedTokenProvider for ServiceAccount {
    #[inline]
    fn get_scoped_token(&self, scopes: crate::Scopes) -> crate::GetTokenFuture<'_> {
        match self.encode_request(scopes) {
            Ok(request) => super::GetTokenFuture::new_https(&self.client, request),
            Err(error) => super::GetTokenFuture::new_error(error),
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
        let Some(path) = std::env::var_os(DEFAULT_ENV_NAME) else {
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

    fn encode_claims(&self, buf: &mut Vec<u8>) -> crate::Result<()> {
        use serde::Serialize;
        self.serialize(&mut serde_json::Serializer::new(buf))?;
        Ok(())
    }
}

// this is a bit overoptimized, but it avoids 2 extra buffer allocations, plus
// overhead from form_urlencoding the JWT. To do this we append the pre-encoded
// form grant_type field + key for the assertion, then encode the JWT directly into
// the end of that buffer, since the JWT shouldn't require any extra urlencoding
fn encode_jwt_body(svc: &ServiceAccount, scopes: Scopes) -> crate::Result<String> {
    const ENCODED_PREFIX: &str =
        "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion=";

    static MAX_LEN: AtomicUsize = AtomicUsize::new(0);

    let est_cap = MAX_LEN.load(Ordering::Relaxed);

    let mut dst = String::with_capacity(if est_cap == 0 {
        ENCODED_PREFIX.len() + 1024
    } else {
        est_cap + 64
    });

    dst.push_str(ENCODED_PREFIX);

    with_buffer(|buf| {
        // encoded header
        buf.clear();
        Header::new(&svc.private_key_id).encode_to(&mut *buf)?;
        URL_SAFE_NO_PAD.encode_string(&buf, &mut dst);
        dst.push('.');

        // encoding claims
        buf.clear();
        Claims::new(svc, scopes).encode_claims(&mut *buf)?;
        URL_SAFE_NO_PAD.encode_string(&buf, &mut dst);

        // Signing + encoding signature. the call to 'sign'
        // handles proper buffer sizing, and it overwrites existing
        // data, so there's no need to do a Vec::clear beforehand
        let jwt = &dst.as_bytes()[ENCODED_PREFIX.len()..];
        svc.signer.sign(jwt, &mut *buf)?;

        // push the separator after signing just to simplify
        // the indexing above, since the signature includes everything
        // up to the separator
        dst.push('.');

        URL_SAFE_NO_PAD.encode_string(&*buf, &mut dst);

        Ok(()) as Result<(), Error>
    })?;

    MAX_LEN.fetch_max(dst.len(), Ordering::Relaxed);

    Ok(dst)
}

// even though we only call 'f' once, using 'impl FnOnce' forces us to do some extra handling
// since we can only optionally call 'f' inside the TLS try_with closure. No caller for this
// function actually needs FnOnce and has trait issues with FnMut, so until then this is fine
fn with_buffer<O>(mut f: impl FnMut(&mut Vec<u8>) -> O) -> O {
    thread_local! {
        static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024));
    }

    let result = BUF.try_with(|ref_cell| match ref_cell.try_borrow_mut() {
        Ok(mut ref_mut) => Some(f(&mut *ref_mut)),
        Err(_) => None,
    });

    match result {
        Ok(Some(output)) => output,
        // if we couldn't get a unique lock on the buffer, or if the TLS is
        // in the middle of being destructed (which would be wierd), just allocate
        // a new temporary buffer since we don't want to panic.
        Ok(None) | Err(_ /* AccessError */) => {
            let mut tmp_buf = Vec::with_capacity(512);
            f(&mut tmp_buf)
        }
    }
}

#[derive(serde::Serialize)]
struct Header<'a> {
    alg: &'a str,
    typ: &'a str,
    kid: &'a str,
}

impl<'a> Header<'a> {
    fn new(kid: &'a str) -> Self {
        Self {
            alg: "RS256",
            typ: "JWT",
            kid,
        }
    }

    fn encode_to(&self, dst: impl std::io::Write) -> Result<(), Error> {
        serde_json::to_writer(dst, self)?;
        Ok(())
    }
}

pin_project_lite::pin_project! {
    pub struct TryLoadFuture<'a> {
        https: CowMut<'a, Option<HttpsClient>>,
        #[pin]
        read_credentials_fut: ReadFuture,
    }
}

impl<'a> TryLoadFuture<'a> {
    pub(crate) fn new_from_path(https: CowMut<'a, Option<HttpsClient>>, path: PathBuf) -> Self {
        Self {
            https,
            read_credentials_fut: ReadFuture::read(path),
        }
    }

    pub(crate) fn new(https: CowMut<'a, Option<HttpsClient>>) -> Option<Self> {
        let Some(path) = std::env::var_os(DEFAULT_ENV_NAME) else {
            return None;
        };

        if path.is_empty() {
            return None;
        }

        Some(Self::new_from_path(https, path.into()))
    }

    pub(crate) fn take_client(&mut self) -> Option<HttpsClient> {
        self.https.take()
    }

    pub(crate) fn take_into_static(self) -> TryLoadFuture<'static> {
        TryLoadFuture {
            https: self.https.take_into_static(),
            read_credentials_fut: self.read_credentials_fut,
        }
    }
}

impl Future for TryLoadFuture<'_> {
    type Output = Result<LoadProviderResult<'static, ServiceAccount>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let bytes = std::task::ready!(this.read_credentials_fut.poll(cx))?;

        let svc = ServiceAccountKey::from_json_bytes(&bytes)?;

        match ServiceAccount::from_parts(&mut this.https, svc) {
            Ok((provider, project_id)) => Poll::Ready(Ok(LoadProviderResult {
                provider,
                project_id,
                token_future: futures::future::TryMaybeDone::Gone,
            })),
            Err((error, client)) => {
                if let Some(client) = client {
                    **this.https = Some(client);
                }

                Poll::Ready(Err(error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ScopedTokenProvider;
    use super::*;

    #[tokio::test]
    async fn test_service_account() -> Result<(), Error> {
        let parts = ServiceAccount::try_load(&mut Default::default())
            .unwrap()
            .await?;

        println!("{:?}", parts.project_id);
        println!("{:#?}", parts.provider);

        let token = parts
            .provider
            .get_scoped_token(Scopes::GCS_READ_ONLY)
            .await?;

        println!("{token:#?}");

        Ok(())
    }
}
