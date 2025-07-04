use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::task::{Context, Poll};

use http::{HeaderName, HeaderValue, Uri};
use tokio::io::{AsyncRead, AsyncWrite};

static HOST: &str = "http://metadata.google.internal";

static TOKEN_URI: LazyLock<Uri> = LazyLock::new(|| {
    Uri::from_static("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
});

static PROJECT_ID_URI: LazyLock<Uri> = LazyLock::new(|| {
    Uri::from_static("http://metadata.google.internal/computeMetadata/v1/project/project-id")
});

const METADATA_FLAVOR_NAME: HeaderName = HeaderName::from_static("metadata-flavor");
const METADATA_FLAVOR_VALUE: HeaderValue = HeaderValue::from_static("Google");

async fn lookup_metadata_host() -> std::io::Result<&'static [SocketAddr]> {
    static SOCKET_ADDRS: tokio::sync::OnceCell<Vec<SocketAddr>> =
        tokio::sync::OnceCell::const_new();

    SOCKET_ADDRS
        .get_or_try_init(|| async {
            let addrs = tokio::net::lookup_host((HOST, 80)).await?;

            Ok(addrs.collect::<Vec<_>>())
        })
        .await
        .map(|vec| vec.as_slice())
}

#[derive(Debug, Clone)]
pub struct MetadataServer {
    client: crate::client::HttpClient,
}

impl MetadataServer {
    /// Initializes a new MetadataServer connection. Returns [`crate::Error::NoProviderFound`]
    /// if the server isn't reachable.
    pub async fn new() -> Result<(Self, crate::ProjectId), crate::Error> {
        let client = crate::client::Client::new_http();

        let Some(project_id) = Self::request_project_id(&client).await? else {
            return Err(crate::Error::NoProviderFound);
        };

        Ok((Self { client }, project_id))
    }

    async fn request_project_id(
        client: &crate::client::HttpClient,
    ) -> Result<Option<crate::ProjectId>, crate::Error> {
        match client.request(make_request(&PROJECT_ID_URI)).await {
            Ok(project_id_bytes) => {
                let project_id_bytes = project_id_bytes.as_ref().trim_ascii();
                let project_id_str = std::str::from_utf8(project_id_bytes).map_err(|_| {
                    crate::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "project id is invalid UTF8: {}",
                            bstr::BStr::new(project_id_bytes)
                        ),
                    ))
                })?;

                Ok(Some(crate::ProjectId(From::from(Arc::from(
                    project_id_str,
                )))))
            }
            // if the metadata server doesnt exist, we expect to get a connection error.
            Err(crate::Error::Hyper(crate::error::HyperError::HyperUtil(err)))
                if err.is_connect() =>
            {
                Ok(None)
            }
            Err(other_err) => Err(other_err),
        }
    }

    async fn try_detect_inner(
        client_opt: &mut Option<crate::client::HttpClient>,
    ) -> crate::Result<Option<(Self, crate::ProjectId)>> {
        let client = client_opt.get_or_insert_with(crate::client::Client::new_http);

        let project_id = match Self::request_project_id(client).await? {
            None => return Ok(None),
            Some(project_id) => project_id,
        };

        let client = client_opt.take().unwrap();

        Ok(Some((Self { client }, project_id)))
    }
}

impl crate::RawTokenProvider for MetadataServer {
    const NAME: &'static str = "metadata server";

    fn try_load(
        ctx: &mut crate::InitContext,
    ) -> impl Future<Output = crate::Result<Option<(Self, crate::ProjectId)>>> + Send + '_ {
        Self::try_detect_inner(&mut ctx.http)
    }

    fn get_token(
        &self,
        _scope: crate::Scopes,
    ) -> impl Future<Output = crate::Result<crate::Token>> + Send + 'static {
        let this = self.clone();
        async move {
            let token = this
                .client
                .request_json::<crate::Token<crate::token::Bearer>>(make_request(&TOKEN_URI))
                .await?;

            Ok(token.into_unit_token_type())
        }
    }
}

fn make_request(uri: &Uri) -> http::Request<crate::client::BytesBody> {
    http::Request::builder()
        .method(http::Method::GET)
        .uri(uri)
        .header(METADATA_FLAVOR_NAME, METADATA_FLAVOR_VALUE)
        .body(crate::client::BytesBody::empty())
        .expect("header/uri values are valid")
}
