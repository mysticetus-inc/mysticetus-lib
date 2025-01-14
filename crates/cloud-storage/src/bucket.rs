use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, ready};

use futures::{Stream, StreamExt};
use gcp_auth_channel::AuthChannel;
use gcp_auth_channel::channel::headers::WithHeaders;
use http::HeaderValue;
use protos::storage::{
    ChecksummedData, ContentRange, ListObjectsRequest, Object, ObjectChecksums, ReadObjectRequest,
    ReadObjectResponse, storage_client,
};

use super::Error;
use crate::get::GetBuilder;
use crate::read::ReadBuilder;

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
            WithHeaders::new(svc, [
                (super::GOOG_PROJ_ID_HEADER, project_id_param),
                (super::GOOG_REQUEST_PARAMS_HEADER, request_params),
            ])
        });

        Self {
            channel,
            qualified_bucket: Arc::from(qualified_bucket),
        }
    }

    pub async fn read_object_to_vec<P>(
        &self,
        path: P,
    ) -> Result<(ReadObjectMetadata, Vec<u8>), Error>
    where
        P: Into<String>,
    {
        let read_object = self.read_object(path.into()).await?;

        let (metadata, mut stream) = read_object.await?;

        let (low, high) = stream.size_hint();

        let mut bytes = Vec::with_capacity(high.unwrap_or(low));

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;

            bytes.extend_from_slice(&chunk);
        }

        Ok((metadata, bytes))
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

    pub(crate) fn into_client(self) -> storage_client::StorageClient<super::HeaderAuthChannel> {
        storage_client::StorageClient::new(self.channel)
    }

    #[inline]
    pub fn read<S>(&self, path: S) -> ReadBuilder<'_, S, (), ()> {
        ReadBuilder::new(self, path)
    }

    #[inline]
    pub fn get<S>(&self, path: S) -> GetBuilder<'_, S, (), ()> {
        GetBuilder::new(self, path)
    }

    pub async fn read_object<P>(&self, path: P) -> Result<ReadObject, Error>
    where
        P: Into<String>,
    {
        let request = ReadObjectRequest {
            bucket: self.qualified_bucket.as_ref().to_owned(),
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

        let mut client = storage_client::StorageClient::new(self.channel.clone());

        let resp = client.read_object(request).await?.into_inner();

        Ok(ReadObject {
            inner: Some(resp),
            check_md5: true,
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
        let _request = ListObjectsRequest {
            parent: self.client.qualified_bucket.as_ref().to_owned(),
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
/*
    let mut client = storage_client::StorageClient::new(
        self.client
            .channel
            .with_scopes(&[Scope::CloudPlatformReadOnly, Scope::GcsReadOnly]),
    );

    let resp = client.list_objects(request).await?.into_inner();
*/
pub struct ObjectStream {
    inner: tonic::Streaming<ReadObjectResponse>,
    next: Option<ChecksummedData>,
    final_checksums: Option<ObjectChecksums>,
    md5: Option<md5::Context>,
    crc32c: u32,
}

pub struct ReadObject {
    inner: Option<tonic::Streaming<ReadObjectResponse>>,
    check_md5: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadObjectMetadata {
    pub object: Object,
    pub range: Option<ContentRange>,
}

impl Future for ReadObject {
    type Output = Result<(ReadObjectMetadata, ObjectStream), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let inner = this
            .inner
            .as_mut()
            .expect("invalid operation: polled after completion");

        let Some(result) = ready!(Pin::new(inner).poll_next(cx)) else {
            return Poll::Ready(Err(Error::Status(tonic::Status::internal(
                "ReadObject stream returned no data",
            ))));
        };

        let inner = this.inner.take().expect("we already know this is Some");

        let first_chunk = result?;

        let object = first_chunk.metadata.ok_or_else(|| {
            Error::Status(tonic::Status::internal(
                "ReadObject metadata expected but not returned",
            ))
        })?;

        let metadata = ReadObjectMetadata {
            object,
            range: first_chunk.content_range,
        };

        let md5 = if this.check_md5
            && metadata
                .object
                .checksums
                .as_ref()
                .is_some_and(|checksums| checksums.md5_hash.len() == 16)
        {
            Some(md5::Context::new())
        } else {
            None
        };

        let stream = ObjectStream {
            next: first_chunk.checksummed_data,
            md5,
            crc32c: 0,
            final_checksums: metadata.object.checksums.clone(),
            inner,
        };

        Poll::Ready(Ok((metadata, stream)))
    }
}

impl ObjectStream {
    fn validate_checksums(&mut self) -> Result<(), Error> {
        let Some(ref checksums) = self.final_checksums else {
            return Ok(());
        };

        match checksums.crc32c {
            Some(0) | None => (),
            Some(crc) if crc == self.crc32c => (),
            Some(_) => {
                return Err(Error::Status(tonic::Status::internal(
                    "ReadObject crc32c check failed",
                )));
            }
        }

        match self.md5.take() {
            Some(md5) => {
                let digest = md5.compute();
                if &digest.0[..] != checksums.md5_hash {
                    return Err(Error::Status(tonic::Status::internal(
                        "ReadObejct md5 check failed",
                    )));
                }
            }
            _ => (),
        }

        Ok(())
    }

    #[inline]
    fn process_chunk(&mut self, chunk: ChecksummedData) -> Result<bytes::Bytes, Error> {
        let ChecksummedData { content, crc32c } = chunk;

        if let Some(ref mut md5) = self.md5 {
            md5.consume(&content);
        }

        match crc32c {
            Some(0) | None => Ok(content),
            Some(crc32) => {
                let chunk_crc = crc32c::crc32c(&content);

                if chunk_crc != crc32 {
                    return Err(Error::Status(tonic::Status::internal(
                        "ReadObject crc32c check failed",
                    )));
                }

                self.crc32c = crc32c::crc32c_combine(self.crc32c, chunk_crc, content.len());
                Ok(content)
            }
        }
    }
}

impl Stream for ObjectStream {
    type Item = Result<bytes::Bytes, Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.next.is_none() {
            self.inner.size_hint()
        } else {
            let (low, high) = self.inner.size_hint();
            (
                low.saturating_add(1),
                high.and_then(|high| high.checked_add(1)),
            )
        }
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(chunk) = this.next.take() {
            // wake immediately so we we can re-register a waker with the actual inner stream
            cx.waker().wake_by_ref();
            return Poll::Ready(Some(this.process_chunk(chunk)));
        }

        loop {
            let chunk = match ready!(Pin::new(&mut this.inner).poll_next(cx)) {
                Some(Ok(chunk)) => chunk,
                Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                None => {
                    this.validate_checksums()?;
                    return Poll::Ready(None);
                }
            };

            if let Some(chunk) = chunk.checksummed_data {
                return Poll::Ready(Some(this.process_chunk(chunk)));
            }
        }
    }
}
