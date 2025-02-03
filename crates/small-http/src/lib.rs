use builder::request::RequestBuilder;
use http::HeaderMap;
mod builder;
pub mod error;
mod io;
pub use error::Error;
use hyper::body::Incoming;
use tokio::task::JoinHandle;

pub mod version;
use url::Url;
pub use version::{Builder, HttpVersion, SendRequest};

pub mod body;
pub mod decompress;
pub mod response;
// pub mod future;
mod reusable_box_future;
mod state;

pub struct Client<B = io::EmptyBody, HttpVers: HttpVersion<B> = version::DefaultHttpVersion> {
    base_uri: Url,
    default_headers: HeaderMap,
    parts: ConnectionParts<B, HttpVers>,
}

struct ConnectionParts<B, HttpVers: HttpVersion<B>> {
    send_request: HttpVers::SendRequest,
    conn: JoinHandle<hyper::Result<()>>,
    request_fut: RawRequestFuture,
}

type RawRequestFuture =
    reusable_box_future::ReusableBoxFuture<'static, hyper::Result<http::Response<Incoming>>>;

impl<B: 'static, HttpVers: HttpVersion<B>> Client<B, HttpVers> {
    async fn new_inner(
        conn_builder: HttpVers::Builder,
        base_uri: Url,
        default_headers: HeaderMap,
    ) -> Result<Self, Error> {
        let host = base_uri
            .host()
            .ok_or_else(|| Error::io_invalid_input("no host"))?;

        let port = base_uri.port().unwrap_or(80);

        let addr = format!("{host}:{port}");

        let io = io::TokioIo::tcp_connect(addr).await?;

        let (send_request, connection) = conn_builder.handshake(io).await?;

        Ok(Self {
            base_uri,
            default_headers,
            parts: ConnectionParts {
                send_request,
                conn: tokio::spawn(connection),
                request_fut: RawRequestFuture::new(std::future::pending()),
            },
        })
    }

    pub fn request(&mut self) -> RequestBuilder<'_, B, HttpVers> {
        let Self {
            base_uri,
            default_headers,
            parts,
        } = self;

        RequestBuilder::new(parts, base_uri, default_headers)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use http::header;
    use io::EmptyBody;
    use version::DefaultHttpVersion;

    use super::*;

    #[tokio::test]
    async fn test_basic_get() -> Result<(), Error> {
        let uri = Url::parse("http://example.com").unwrap();

        let mut client = Client::<EmptyBody, DefaultHttpVersion>::new_inner(
            <<DefaultHttpVersion as HttpVersion<EmptyBody>>::Builder as Builder<EmptyBody>>::new(),
            uri,
            HeaderMap::new(),
        )
        .await?;

        let response = client
            .request()
            .header(header::ACCEPT_ENCODING, "gzip")
            .get(EmptyBody)
            .await?;

        let (parts, body) = response.into_parts();

        println!("{parts:#?}");

        let mut stream = std::pin::pin!(crate::body::BodyStream::new(body));

        let mut stdout = std::io::stdout();

        use futures::StreamExt;

        let mut len = 0;
        let mut chunks = 0;

        while let Some(result) = stream.next().await {
            let bytes = result?;
            len += bytes.len();
            stdout.write_all(&bytes)?;
            chunks += 1;
        }

        println!("got {len} bytes in {chunks} chunks");

        Ok(())
    }
}
