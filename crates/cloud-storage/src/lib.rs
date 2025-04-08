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
pub mod write;

const GOOG_PROJ_ID_HEADER: HeaderName = HeaderName::from_static("x-goog-project-id");
const GOOG_REQUEST_PARAMS_HEADER: HeaderName = HeaderName::from_static("x-goog-request-params");

/// alias to the wrapped service that adds the required headers.
pub(crate) type HeaderAuthChannel<Svc = Channel> = AuthChannel<WithHeaders<Svc, Http>>;

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use rand::RngCore;

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

    #[tokio::test]
    async fn test_write() -> Result<()> {
        const PATH: &str = "__test-read-write-object";
        const CONTENT_LEN: usize = 128;
        const BUCKET: &str = "staging.mysticetus-oncloud.appspot.com";

        let mut buf = vec![0; CONTENT_LEN];

        let mut rng = rand::rng();
        rng.fill_bytes(&mut buf);

        let content = Bytes::from(buf);

        let mut client =
            StorageClient::new("mysticetus-oncloud", gcp_auth_channel::Scope::GcsReadWrite)
                .await?
                .into_bucket(BUCKET);

        let object = client.write(PATH).write_bytes(content.clone()).await?;

        println!("{object:#?}");
        assert_eq!(object.size as usize, CONTENT_LEN);

        let (read_obj, read_bytes) = client
            .read(PATH)
            .generation(object.generation as u64)
            .stream()
            .await?
            .collect_to_vec()
            .await?;

        // we cant compare all fields in both objects, since google inserts extra values when
        // reading, so just compare a bunch of obvious ones that should be identical.
        assert_eq!(read_obj.name, object.name);
        assert_eq!(read_obj.bucket, object.bucket);
        assert_eq!(read_obj.generation, object.generation);
        assert_eq!(read_obj.metadata, object.metadata);
        assert_eq!(read_obj.size, object.size);
        assert_eq!(read_obj.checksums, object.checksums);
        assert_eq!(read_obj.finalize_time, object.finalize_time);
        assert_eq!(read_obj.etag, object.etag);

        assert_eq!(content.as_ref(), read_bytes.as_slice());

        Ok(())
    }
}
