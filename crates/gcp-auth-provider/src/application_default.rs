use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::{BufMut, BytesMut};
use http::HeaderValue;

use crate::client::BytesBody;
use crate::{Error, Scopes};

const JSON_TYPE: HeaderValue = HeaderValue::from_static("application/json");

pub struct ApplicationDefault {
    client: crate::client::HttpsClient,
    delegates: Vec<Box<str>>,
    service_account_url: Option<Box<str>>,
    client_id: Box<str>,
    client_secret: Box<str>,
    refresh_token: Box<str>,
}

impl fmt::Debug for ApplicationDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApplicationDefault")
            .field("client_id", &self.client_id)
            .finish_non_exhaustive()
    }
}

impl ApplicationDefault {
    fn make_request(&self, scope: Scopes) -> Result<http::Request<BytesBody>, Error> {
        static MAX_ENCODED_CAPACITY: AtomicUsize = AtomicUsize::new(0);

        let cap = MAX_ENCODED_CAPACITY.load(Ordering::Relaxed);

        let mut bytes = BytesMut::with_capacity(if cap == 0 { 512 } else { cap });

        let uri = match self.service_account_url.as_deref() {
            Some(uri) => {
                let impersonate_request = ImpersonateRequest {
                    delegates: &self.delegates,
                    scope,
                };
                serde_json::to_writer((&mut bytes).writer(), &impersonate_request)?;
                uri
            }
            None => {
                let refresh_request = RefreshRequest {
                    client_id: &self.client_id,
                    client_secret: &self.client_secret,
                    refresh_token: &self.refresh_token,
                    grant_type: "refresh_token",
                };
                serde_json::to_writer((&mut bytes).writer(), &refresh_request)?;
                "https://accounts.google.com/o/oauth2/token"
            }
        };

        MAX_ENCODED_CAPACITY.fetch_max(bytes.len(), Ordering::Relaxed);

        println!("req: {}", bstr::BStr::new(&bytes));

        http::Request::builder()
            .method(http::Method::POST)
            .uri(uri)
            .header(http::header::CONTENT_TYPE, JSON_TYPE)
            .body(BytesBody::new(bytes.freeze()))
            .map_err(|err| {
                crate::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            })
    }
}

impl super::RawTokenProvider for ApplicationDefault {
    const NAME: &'static str = "application default";

    fn try_load(
        ctx: &mut crate::InitContext,
    ) -> impl Future<Output = crate::Result<Option<(Self, crate::ProjectId)>>> + Send + '_ {
        async move {
            let Some(mut file) = dirs::config_dir() else {
                return Ok(None);
            };

            file.push("gcloud/application_default_credentials.json");

            let bytes = match tokio::fs::read(&file).await {
                Ok(bytes) => bytes,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(err) => return Err(err.into()),
            };

            let (delegates, service_account_url, creds) =
                if memchr::memmem::find(&bytes, "\"source_credentials\"".as_bytes()).is_some() {
                    let ImpersonatedCredentials {
                        source_credentials,
                        service_account_impersonation_url,
                        delegates,
                    } = path_aware_serde::json::deserialize_slice(&bytes)?;
                    (
                        delegates,
                        Some(Box::from(service_account_impersonation_url)),
                        source_credentials,
                    )
                } else {
                    let creds: ApplicationCredentials =
                        path_aware_serde::json::deserialize_slice(&bytes)?;

                    (Vec::new(), None, creds)
                };

            let proj_id: Arc<str> = match (
                service_account_url.as_deref(),
                creds.quota_project_id.as_ref(),
            ) {
                (_, Some(project_id)) => Arc::from(&**project_id),
                (Some(uri), None) => match find_project_id_in_impersonated_uri(uri) {
                    Some(project_id) => Arc::from(project_id),
                    None => {
                        return Err(Error::Io(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "no project id found",
                        )));
                    }
                },
                (None, None) => {
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "no project id found",
                    )));
                }
            };

            let client = match ctx.https.take() {
                Some(client) => client,
                None => crate::client::Client::new_https()?,
            };

            let new = Self {
                client,
                delegates,
                service_account_url,
                client_id: Box::from(creds.client_id),
                client_secret: Box::from(creds.client_secret),
                refresh_token: Box::from(creds.refresh_token),
            };

            Ok(Some((new, crate::ProjectId(From::from(proj_id)))))
        }
    }

    fn get_token(
        &self,
        scopes: crate::Scopes,
    ) -> impl Future<Output = crate::Result<crate::Token>> + Send + 'static {
        let request_result = self.make_request(scopes);
        let client = self.client.clone();

        async move {
            let request = request_result?;
            client.request_json(request).await
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "'de: 'a")]
struct ApplicationCredentials<'a> {
    #[serde(with = "serde_helpers::borrow")]
    client_id: Cow<'a, str>,
    #[serde(with = "serde_helpers::borrow")]
    client_secret: Cow<'a, str>,
    #[serde(default, with = "serde_helpers::borrow::optional")]
    quota_project_id: Option<Cow<'a, str>>,
    #[serde(with = "serde_helpers::borrow")]
    refresh_token: Cow<'a, str>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "'de: 'a")]
struct ImpersonatedCredentials<'a> {
    #[serde(default)]
    delegates: Vec<Box<str>>,
    service_account_impersonation_url: Cow<'a, str>,
    source_credentials: ApplicationCredentials<'a>,
}

#[derive(serde::Serialize)]
struct RefreshRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    grant_type: &'a str,
    refresh_token: &'a str,
}

#[derive(serde::Serialize)]
struct ImpersonateRequest<'a> {
    delegates: &'a Vec<Box<str>>,
    #[serde(serialize_with = "crate::scope::serialize_scope_urls_as_array")]
    scope: Scopes,
}

fn find_project_id_in_impersonated_uri(uri: &str) -> Option<&str> {
    let start = memchr::memchr(b'@', uri.as_bytes())? + 1;
    let remaining = uri.get(start..)?;

    let len = memchr::memchr(b'.', remaining.as_bytes())?;

    remaining.get(..len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RawTokenProvider, Scopes};

    #[tokio::test]
    async fn test_application_default() -> Result<(), Error> {
        let (app, proj_id) = ApplicationDefault::try_load(&mut Default::default())
            .await?
            .unwrap();
        println!("{proj_id:?}");

        let token = app.get_token(Scopes::GCS_READ_ONLY).await?;
        println!("{token:#?}");
        Ok(())
    }
}
