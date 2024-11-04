//! Currently unusable.
//!
//! The GCS gRPC API is still in internal testing, so any and all requests (to buckets that arent
//! a part of their testing) return a permissions error with the message:
//!
//! Requested bucket, '#######', is not allowed to access the GCS gRPC API. Note: this API is
//! currently in testing, and is not yet available for general use."
#![allow(unused)]

use std::time::Duration;

use gcp_auth_channel::channel::headers::{Http, WithHeader};
use gcp_auth_channel::{Auth, AuthChannel};
use tonic::transport::{Channel, ClientTlsConfig};

pub mod bucket;

pub mod error;
pub use error::Error;

const ADMIN_SCOPE: &str = "https://www.googleapis.com/auth/devstorage.full_control";
const READ_ONLY_SCOPE: &str = "https://www.googleapis.com/auth/devstorage.read_only";
const READ_WRITE_SCOPE: &str = "https://www.googleapis.com/auth/devstorage.read_write";

const STORAGE_DOMAIN: &str = "storage.googleapis.com";
const STORAGE_URL: &str = "https://storage.googleapis.com";

const GOOG_PROJ_ID_HEADER: &str = "x-goog-project-id";

const DEFUALT_KEEPALIVE_DUR: Duration = Duration::from_secs(60);

/// alias to the internal intercepted service.
pub(crate) type HeaderAuthChannel = AuthChannel<WithHeader<Channel, Http>>;

#[cfg(test)]
mod tests {
    fn test_bounds<S, Req, Res>()
    where
        S: tower::Service<http::Request<Req>, Response = http::Response<Res>>,
    {
    }

    #[test]
    fn test() {
        // test_bounds::<super::HeaderAuthChannel, (), tonic::body::BoxBody>()
    }
}

async fn build_channel() -> Result<Channel, Error> {
    Channel::from_static(STORAGE_URL)
        .user_agent("cloud-storage-rs")?
        .tcp_keepalive(Some(DEFUALT_KEEPALIVE_DUR))
        .tls_config(
            ClientTlsConfig::new()
                .domain_name(STORAGE_DOMAIN)
                .with_native_roots(),
        )?
        .connect()
        .await
        .map_err(Error::from)
}

async fn build_auth(
    project_id: &'static str,
    scope: gcp_auth_channel::Scope,
) -> Result<Auth, Error> {
    Auth::new(project_id, scope).await.map_err(Error::from)
}

#[derive(Debug, Clone)]
pub struct StorageClient {
    channel: HeaderAuthChannel,
}

impl StorageClient {
    pub async fn new(
        project_id: &'static str,
        scope: gcp_auth_channel::Scope,
    ) -> Result<Self, Error> {
        let value = http::HeaderValue::from_static(project_id);

        let (auth, channel) = tokio::try_join!(build_auth(project_id, scope), build_channel())?;

        let auth_channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build();

        let channel =
            gcp_auth_channel::channel::headers::WithHeaderBuilder::from_service(auth_channel)
                .static_key(GOOG_PROJ_ID_HEADER)
                .value(value);

        Ok(Self { channel })
    }

    pub async fn from_service_account<P>(
        project_id: &'static str,
        scope: gcp_auth_channel::Scope,
        path: P,
    ) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path>,
    {
        let value = http::HeaderValue::from_static(project_id);

        let auth = Auth::new_from_service_account_file(project_id, path.as_ref(), scope)?;
        let channel = build_channel().await?;

        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth(auth)
            .build()
            .attach_header()
            .static_key(GOOG_PROJ_ID_HEADER)
            .value(value);

        Ok(Self { channel })
    }

    pub fn bucket<B>(&self, bucket: B) -> bucket::BucketClient
    where
        B: Into<String>,
    {
        bucket::BucketClient::new(self.channel.clone(), bucket.into())
    }
}

// #[ignore = "GCS gRPC API not public yet"]
#[tokio::test]
async fn test_read() -> Result<(), Error> {
    let client =
        StorageClient::new("mysticetus-oncloud", gcp_auth_channel::Scope::GcsReadOnly).await?;

    let bucket = client.bucket("mysticetus-replicated-data");

    let bytes = match bucket
        .read_object_to_vec(
            "Testing4-101112/Edits/PaulTestVis/2022-03-22/\
             PaulTestVis-2022-03-22-1629-Final-Edited-DS.Mysticetus",
        )
        .await
    {
        Ok(b) => b,
        Err(error) => {
            println!("{error:#?}");
            return Ok(());
        }
    };

    std::fs::write("test-file.zip", bytes).unwrap();

    Ok(())
}
