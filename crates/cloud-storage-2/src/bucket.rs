use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, ready};

use futures::{Stream, StreamExt};
use gcp_auth_channel::Scope;
use protos::storage::{
    ListObjectsRequest, Object, ReadObjectRequest, ReadObjectResponse, storage_client,
};

use super::{Error, HeaderAuthChannel};

#[derive(Debug, Clone)]
pub struct BucketClient {
    bucket: Arc<String>,
    channel: HeaderAuthChannel,
}

impl BucketClient {
    pub(crate) fn new(channel: HeaderAuthChannel, bucket: String) -> Self {
        Self {
            channel,
            bucket: Arc::new(bucket),
        }
    }

    pub async fn read_object_to_vec<P>(&self, path: P) -> Result<Vec<u8>, Error>
    where
        P: Into<String>,
    {
        let mut stream = self.read_object(path.into()).await?;

        let (low, high) = stream.size_hint();

        let mut bytes = Vec::with_capacity(high.unwrap_or(low));

        while let Some(chunk_result) = stream.next().await {
            let mut chunk = chunk_result?;

            bytes.extend_from_slice(&chunk);
        }

        println!("{stream:#?}");

        Ok(bytes)
    }

    pub async fn read_object<P>(&self, path: P) -> Result<ObjectStream, Error>
    where
        P: Into<String>,
    {
        let request = ReadObjectRequest {
            bucket: format!(
                "projects/{}/buckets/{}",
                self.channel.auth().project_id(),
                self.bucket
            ),
            object: path.into(),
            generation: 0,
            read_mask: None,
            read_limit: 0,
            read_offset: 0,
            if_generation_match: None,
            if_generation_not_match: None,
            if_metageneration_match: None,
            if_metageneration_not_match: None,
            common_object_request_params: None,
        };

        let clone = self.channel.clone();
        let mut client =
            storage_client::StorageClient::new(clone.with_scope(Scope::CloudPlatformReadOnly));

        let resp = client.read_object(request).await?.into_inner();

        Ok(ObjectStream {
            inner: resp,
            md5_ctx: md5::Context::new(),
            crc_ctx: crc32fast::Hasher::new(),
            object: None,
        })
    }

    pub fn list(&self) -> ListRequestBuilder {
        ListRequestBuilder::new(self.clone())
    }

    pub async fn list_prefix<P>(&self, prefix: P) -> Result<(), Error>
    where
        P: Into<String>,
    {
        self.list().prefix(prefix.into()).get().await
    }
}

#[derive(Debug, Clone)]
pub struct ListRequestBuilder {
    client: BucketClient,
    delimiter: Option<String>,
    include_trailing_delimiter: bool,
    prefix: Option<String>,
    lexicographic_start: Option<String>,
    lexicographic_end: Option<String>,
}

impl ListRequestBuilder {
    fn new(client: BucketClient) -> Self {
        Self {
            client,
            delimiter: None,
            include_trailing_delimiter: false,
            prefix: None,
            lexicographic_end: None,
            lexicographic_start: None,
        }
    }

    pub fn prefix<P>(mut self, prefix: P) -> Self
    where
        P: Into<String>,
    {
        self.prefix = Some(prefix.into());
        self
    }

    pub async fn get(self) -> Result<(), Error> {
        let parent = format!(
            "projects/{}/buckets/{}",
            self.client.channel.auth().project_id(),
            self.client.bucket
        );

        let _request = ListObjectsRequest {
            parent,
            page_size: 1000,
            soft_deleted: false,
            page_token: String::new(),
            delimiter: self.delimiter.unwrap_or_default(),
            include_trailing_delimiter: self.include_trailing_delimiter,
            prefix: self.prefix.unwrap_or_default(),
            versions: false,
            read_mask: None,
            lexicographic_end: self.lexicographic_end.unwrap_or_default(),
            lexicographic_start: self.lexicographic_start.unwrap_or_default(),
            match_glob: String::new(),
            include_folders_as_prefixes: false,
        };

        /*
            let mut client = storage_client::StorageClient::new(
                self.client
                    .channel
                    .with_scopes(&[Scope::CloudPlatformReadOnly, Scope::GcsReadOnly]),
            );

            let resp = client.list_objects(request).await?.into_inner();
        */
        Ok(())
    }
}

pub struct ObjectStream {
    inner: tonic::Streaming<ReadObjectResponse>,
    md5_ctx: md5::Context,
    crc_ctx: crc32fast::Hasher,
    object: Option<Object>,
}

impl fmt::Debug for ObjectStream {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("ObjectStream")
            .field("inner", &self.inner)
            .field("md5_ctx", &"...")
            .field("crc_ctx", &self.crc_ctx)
            .field("object", &self.object)
            .finish()
    }
}

impl Stream for ObjectStream {
    type Item = Result<bytes::Bytes, Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut_self = self.get_mut();

        let chunk = match ready!(Pin::new(&mut mut_self.inner).poll_next(cx)) {
            Some(Ok(chunk)) => chunk,
            Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
            None => return Poll::Ready(None),
        };

        println!("raw_response {chunk:#?}");

        if let Some(object) = chunk.metadata {
            mut_self.object = Some(object);
        }

        if let Some(checksum) = chunk.object_checksums {
            mut_self.md5_ctx.consume(checksum.md5_hash.as_ref());
        }

        let data = match chunk.checksummed_data {
            Some(data) => data,
            None => return Poll::Pending,
        };

        if let Some(crc32) = data.crc32c {
            let mut hasher = mut_self.crc_ctx.clone();
            hasher.reset();
            hasher.update(data.content.as_ref());

            if hasher.finalize() != crc32 {
                println!("crc32 hash doesn't match!");
            }
        }

        Poll::Ready(Some(Ok(data.content)))
    }
}
