use std::io;
use std::pin::Pin;
use std::sync::LazyLock;
use std::task::{Context, Poll};

use http::{HeaderName, HeaderValue, Uri};
use hyper::client::conn::http1::SendRequest;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task::JoinHandle;

pub mod error;
pub use error::Error;

static TOKEN_URI: LazyLock<Uri> = LazyLock::new(|| {
    Uri::from_static("/computeMetadata/v1/instance/service-accounts/default/token")
});

static PROJECT_ID_URI: LazyLock<Uri> =
    LazyLock::new(|| Uri::from_static("/computeMetadata/v1/project/project-id"));

const METADATA_FLAVOR_NAME: HeaderName = HeaderName::from_static("metadata-flavor");
const METADATA_FLAVOR_VALUE: HeaderValue = HeaderValue::from_static("google");

pub struct MetadataServer {
    driver: JoinHandle<hyper::Result<()>>,
    send_req: SendRequest<Empty>,
}

impl MetadataServer {
    pub async fn new() -> Result<Self, Error> {
        let host = tokio::net::lookup_host("metadata.google.internal:80")
            .await?
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "no hosts found for the internal metadata server",
                )
            })?;
        let stream = tokio::net::TcpStream::connect(host).await?;

        let (send_req, conn) = hyper::client::conn::http1::handshake(TokioIo { stream }).await?;

        Ok(Self {
            send_req,
            driver: tokio::spawn(conn),
        })
    }
}

#[tokio::test]
async fn test_metadata() -> Result<(), Error> {
    let server = MetadataServer::new().await?;

    Ok(())
}
