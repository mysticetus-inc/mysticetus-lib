use std::sync::Arc;

use gcp_auth_channel::AuthChannel;
use gcp_auth_channel::channel::headers::WithHeaders;
use http::HeaderValue;
use protos::storage::{Object, storage_client};

use super::Error;
use crate::get::GetBuilder;
use crate::read::ReadBuilder;
use crate::write::WriteBuilder;

#[derive(Debug, Clone)]
pub struct BucketClient {
    qualified_bucket: Arc<str>,
    channel: super::HeaderAuthChannel,
}

impl BucketClient {
    pub(crate) fn new(channel: AuthChannel, bucket: &str) -> Self {
        let qualified_bucket = format!("projects/_/buckets/{bucket}");

        let project_id_param =
            HeaderValue::from_str(channel.auth().project_id()).expect("invalid project_id");

        let request_params = HeaderValue::from_str(&format!("bucket={}", qualified_bucket))
            .expect("invalid bucket name");

        let channel = channel.wrap_service(|svc| {
            WithHeaders::new(
                svc,
                [
                    (super::GOOG_PROJ_ID_HEADER, project_id_param),
                    (super::GOOG_REQUEST_PARAMS_HEADER, request_params),
                ],
            )
        });

        Self {
            channel,
            qualified_bucket: Arc::from(qualified_bucket),
        }
    }

    pub async fn read_object_to_vec<P>(&mut self, path: P) -> Result<(Object, Vec<u8>), Error>
    where
        P: Into<String>,
    {
        self.read(path.into())
            .stream()
            .await?
            .collect_to_vec()
            .await
    }

    pub(crate) fn qualified_bucket(&self) -> &str {
        self.qualified_bucket.as_ref()
    }

    pub(crate) fn client(&self) -> storage_client::StorageClient<super::HeaderAuthChannel> {
        storage_client::StorageClient::new(self.channel.clone())
    }

    pub(crate) fn client_mut(
        &mut self,
    ) -> storage_client::StorageClient<&mut super::HeaderAuthChannel> {
        storage_client::StorageClient::new(&mut self.channel)
    }

    #[inline]
    pub fn read<S>(&mut self, path: S) -> ReadBuilder<'_, S, (), ()> {
        ReadBuilder::new(self, path)
    }

    #[inline]
    pub fn write(&mut self, path: impl Into<String>) -> WriteBuilder<'_> {
        WriteBuilder::new(self, path.into(), crate::write::NonResumable)
    }

    #[inline]
    pub fn get<S>(&mut self, path: S) -> GetBuilder<'_, S, (), ()> {
        GetBuilder::new(self, path)
    }

    pub fn list(&mut self) -> crate::list::ListBuilder<'_> {
        crate::list::ListBuilder::new(self)
    }
}
