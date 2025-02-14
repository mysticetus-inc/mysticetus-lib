use gcp_auth_channel::AuthChannel;
use gcp_auth_channel::channel::headers::{Http, WithHeaders};
use http::HeaderName;
use tonic::transport::Channel;

mod client;
pub use client::StorageClient;

mod bucket;
pub use bucket::BucketClient;

pub mod error;
pub use error::Error;

pub mod generation;
pub mod get;
pub mod list;
pub mod read;
pub mod util;
// TODO: writes
// pub mod write;

const GOOG_PROJ_ID_HEADER: HeaderName = HeaderName::from_static("x-goog-project-id");
const GOOG_REQUEST_PARAMS_HEADER: HeaderName = HeaderName::from_static("x-goog-request-params");

/// alias to the wrapped service that adds the required headers.
pub(crate) type HeaderAuthChannel<Svc = Channel> = AuthChannel<WithHeaders<Svc, Http>>;

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_client() -> Result<BucketClient> {
        async fn init() -> Result<BucketClient> {
            StorageClient::new("mysticetus-oncloud", gcp_auth_channel::Scope::GcsReadOnly)
                .await
                .map(|client| client.into_bucket("mysticetus-replicated-data"))
        }

        static CLIENT: tokio::sync::OnceCell<BucketClient> = tokio::sync::OnceCell::const_new();

        CLIENT.get_or_try_init(init).await.cloned()
    }

    #[tokio::test]
    async fn test_list_prefix() -> Result<()> {
        const DIR: &str = "Testing4-101112/Edits/";

        let mut client = get_client().await?;

        let results = client.list().prefix(DIR).folders().collect().await?;

        println!("{results:#?}");

        assert!(
            results
                .prefixes
                .contains(&"Testing4-101112/Edits/PaulTestVis/".to_owned())
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_read() -> Result<()> {
        const PATH: &str = "Testing4-101112/Edits/PaulTestVis/2022-03-22/\
                            PaulTestVis-2022-03-22-1629-Final-Edited-DS.Mysticetus";

        let mut client = get_client().await?;

        let read_stream = client.read(PATH).range(-3000..)?.stream().await?;
        println!("{read_stream:#?}");

        let (object, bytes) = read_stream.collect_to_vec().await?;
        println!("{object:#?}");

        assert_eq!(bytes.len(), 3000);

        Ok(())
    }
}
