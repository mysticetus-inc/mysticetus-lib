use std::sync::{Arc, Mutex};

use http::{HeaderMap, Uri};
mod builder;
pub mod error;
mod io;
pub use error::Error;
use hyper::body::Incoming;
use hyper::client::conn::http1::{self, SendRequest};
use tokio::task::JoinHandle;

pub struct Client<B = io::EmptyBody> {
    base_uri: Uri,
    default_headers: HeaderMap,
    send_request: SendRequest<B>,
    conn: JoinHandle<hyper::Result<()>>,
}

impl<B> Client<B>
where
    B: http_body::Body + Send + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    pub async fn new(base_uri: Uri, default_headers: HeaderMap) -> Result<Self, Error> {
        let host = base_uri
            .host()
            .ok_or_else(|| Error::io_invalid_input("no host"))?;

        let port = base_uri.port_u16().unwrap_or(80);

        let addr = format!("{host}:{port}");

        let io = io::TokioIo::tcp_connect(addr).await?;

        let (send_request, connection) = http1::handshake::<io::TokioIo, B>(io).await?;

        Ok(Self {
            send_request,
            base_uri,
            default_headers,
            conn: tokio::spawn(connection),
        })
    }

    async fn send_request(
        &mut self,
        request: http::Request<B>,
    ) -> Result<http::Response<Incoming>, Error> {
        self.send_request.ready().await?;

        self.send_request
            .send_request(request)
            .await
            .map_err(Error::Hyper)
    }
}
