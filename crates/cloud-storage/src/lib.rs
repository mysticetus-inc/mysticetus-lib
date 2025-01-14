use gcp_auth_channel::AuthChannel;
use gcp_auth_channel::channel::headers::{Http, WithHeaders};
use http::HeaderName;
use tonic::transport::Channel;

mod client;
pub use client::StorageClient;

pub mod bucket;

pub mod error;
pub use error::Error;

pub mod generation;
pub mod get;
pub mod list;
pub mod read;
pub mod write;

const GOOG_PROJ_ID_HEADER: HeaderName = HeaderName::from_static("x-goog-project-id");
const GOOG_REQUEST_PARAMS_HEADER: HeaderName = HeaderName::from_static("x-goog-request-params");

/// alias to the wrapped service that adds the required headers.
pub(crate) type HeaderAuthChannel<Svc = Channel> = AuthChannel<WithHeaders<Svc, Http>>;

pub type Result<T> = core::result::Result<T, Error>;

#[tokio::test]
async fn test_read() -> Result<()> {
    let client = StorageClient::new("mysticetus-oncloud", gcp_auth_channel::Scope::GcsReadOnly)
        .await?
        .into_bucket("mysticetus-replicated-data");

    let (metadata, bytes) = match client
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

    println!("{metadata:#?}");

    std::fs::write("test-file.zip", bytes).unwrap();

    Ok(())
}
