use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes};
use net_utils::bidi2::{self, RequestSink};
use protos::storage::write_object_request::{Data, FirstMessage};
use protos::storage::write_object_response::WriteStatus;
use protos::storage::{
    CommonObjectRequestParams, Object, ObjectChecksums, WriteObjectRequest, WriteObjectResponse,
    WriteObjectSpec,
};
use tokio::task::JoinHandle;

use crate::Error;
use crate::bucket::BucketClient;

pub struct WriteBuilder<'a, Kind = NonResumable> {
    client: &'a BucketClient,
    common_object_request_params: Option<CommonObjectRequestParams>,
    spec: WriteObjectSpec,
    write_offset: i64,
    compute_checksums: bool,
    kind: Kind,
}

pub struct Resumable;
pub struct NonResumable;

impl<'a, Kind> WriteBuilder<'a, Kind> {
    pub(crate) fn new(client: &'a BucketClient, path: String, kind: Kind) -> Self {
        Self {
            client,
            write_offset: 0,
            compute_checksums: true,
            spec: WriteObjectSpec {
                resource: Some(Object {
                    name: path,
                    bucket: client.qualified_bucket().to_owned(),
                    ..Default::default()
                }),
                appendable: None,
                predefined_acl: String::new(),
                if_generation_match: None,
                if_generation_not_match: None,
                if_metageneration_match: None,
                if_metageneration_not_match: None,
                object_size: None,
            },
            common_object_request_params: None,
            kind,
        }
    }
}

impl<'a> WriteBuilder<'a, NonResumable> {
    pub async fn write_buf<B: Buf>(mut self, mut buf: B) -> crate::Result<Object> {
        // if the buffer is contiguous (most are), just
        // use a single call to copy_to_bytes (since many Buf types have an
        // optimized version that doesn't actually copy memory, minus Vec<u8>).
        if buf.remaining() == buf.chunk().len() {
            let bytes = buf.copy_to_bytes(buf.remaining());
            return self.write_bytes(bytes).await;
        }

        self.spec.object_size = Some(buf.remaining() as i64);

        let mut write_offset = self.write_offset;

        let mut first_message = Some(FirstMessage::WriteObjectSpec(self.spec));

        let mut total_crc32c = 0;

        let (sink, stream) = bidi2::build_pair();

        let mut client = self.client.client();

        // drive the actual request in a new task, since we want requests to actually be sent while
        // we're building them
        let response_handle = tokio::spawn(async move { client.write_object(stream).await });

        while buf.has_remaining() {
            // copy by chunk size, since non-contiguous buffers are usually
            // something like VecDeque<Bytes>
            let chunk_size = buf.chunk().len();
            let content = buf.copy_to_bytes(chunk_size);

            let chunk_crc32c = if self.compute_checksums {
                let chunk_crc = crc32c::crc32c(&content);
                total_crc32c = crc32c::crc32c_combine(total_crc32c, chunk_crc, content.len());
                Some(chunk_crc)
            } else {
                None
            };

            let object_checksums =
                (self.compute_checksums && !buf.has_remaining()).then(|| ObjectChecksums {
                    crc32c: Some(total_crc32c),
                    md5_hash: Bytes::new(),
                });

            let request = WriteObjectRequest {
                write_offset,
                object_checksums,
                finish_write: !buf.has_remaining(),
                common_object_request_params: self.common_object_request_params.take(),
                first_message: first_message.take(),
                data: Some(Data::ChecksummedData(protos::storage::ChecksummedData {
                    crc32c: chunk_crc32c,
                    content,
                })),
            };

            write_offset += chunk_size as i64;

            if sink.send(request).is_err() {
                drop(sink);
                // if the reciever died, the request should be done (and dead).
                // If there was no error in the JoinHandle for whatever reason,
                // ensure we send back a data loss status
                response_handle.await??;

                return Err(tonic::Status::data_loss(
                    "write failed successfully with bytes remaining?",
                )
                .into());
            }
        }

        // drop the sink that way the request stream knows we're done
        drop(sink);

        let write_response = response_handle.await??.into_inner();

        extract_object(write_response)
    }

    pub async fn write_bytes(mut self, content: Bytes) -> crate::Result<Object> {
        let crc32c = self.compute_checksums.then(|| crc32c::crc32c(&content));

        self.spec.object_size = Some(content.len() as i64);

        let request = WriteObjectRequest {
            write_offset: self.write_offset,
            finish_write: true,
            object_checksums: crc32c.map(|crc32c| protos::storage::ObjectChecksums {
                crc32c: Some(crc32c),
                md5_hash: Bytes::new(),
            }),
            common_object_request_params: self.common_object_request_params,
            first_message: Some(FirstMessage::WriteObjectSpec(self.spec)),
            data: Some(Data::ChecksummedData(protos::storage::ChecksummedData {
                content,
                crc32c,
            })),
        };

        let write_response = self
            .client
            .client()
            .write_object(futures::stream::once(std::future::ready(request)))
            .await?
            .into_inner();

        extract_object(write_response)
    }
}

fn extract_object(response: WriteObjectResponse) -> crate::Result<Object> {
    match response.write_status {
        Some(WriteStatus::Resource(obj)) => Ok(obj),
        Some(WriteStatus::PersistedSize(size)) => Err(Error::internal(format!(
            "expected a completed write, got an intermediate write result of {size} bytes"
        ))),
        None => Err(Error::internal("got an empty WriteObjectResponse")),
    }
}

pub enum PredefinedAcl {
    AuthenticatedRead,
    BucketOwnerFullControl,
    BucketOwnerRead,
    Private,
    ProjectPrivate,
    PublicRead,
}

pub struct WriteSink {
    handle: JoinHandle<tonic::Result<WriteObjectResponse>>,
    sink: RequestSink<WriteObjectRequest>,
    write_offset: i64,
    overall_crc32c: u32,
    state: WriteStateFlags,
    first_message: Box<Option<FirstMessage>>,
    common_params: Option<CommonObjectRequestParams>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct WriteStateFlags: u8 {
        const FINISHED_WRITE = 1;
        const COMPUTE_CHECKSUMS = 2;
    }
}

impl WriteSink {
    fn build_next_message(&mut self, finish_write: bool, content: Bytes) -> WriteObjectRequest {
        if self.state.contains(WriteStateFlags::FINISHED_WRITE) {
            panic!("WriteSink send called after the final chunk was already sent");
        }

        let curr_write_offset = self.write_offset;
        self.write_offset += content.len() as i64;

        if finish_write {
            self.state.insert(WriteStateFlags::FINISHED_WRITE);
        }

        let mut object_checksums = None;
        let mut crc32c = None;
        if self.state.contains(WriteStateFlags::COMPUTE_CHECKSUMS) {
            let chunk_crc = crc32c::crc32c(&content);
            crc32c = Some(chunk_crc);

            // update the overall crc
            self.overall_crc32c =
                crc32c::crc32c_combine(self.overall_crc32c, chunk_crc, content.len());

            if finish_write {
                object_checksums = Some(ObjectChecksums {
                    crc32c: Some(self.overall_crc32c),
                    md5_hash: Bytes::new(),
                });
            }
        }

        WriteObjectRequest {
            write_offset: curr_write_offset,
            object_checksums,
            finish_write,
            common_object_request_params: self.common_params.take(),
            first_message: self.first_message.take(),
            data: Some(Data::ChecksummedData(protos::storage::ChecksummedData {
                content,
                crc32c,
            })),
        }
    }

    pub fn write_bytes(&mut self, last_chunk: bool, bytes: Bytes) -> Result<(), Error> {
        let request = self.build_next_message(last_chunk, bytes);
        match self.sink.send(request) {
            Ok(()) => Ok(()),
            Err(_) => {
                // if the reciever hung up, we can never finish, so ensure this is unset.
                self.state.remove(WriteStateFlags::FINISHED_WRITE);

                Err(Error::Status(tonic::Status::data_loss(
                    "request handler failed",
                )))
            }
        }
    }
}

impl Future for WriteSink {
    type Output = crate::Result<Object>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let result = std::task::ready!(Pin::new(&mut this.handle).poll(cx));
        let response = result??;

        // if there were no errors but we didnt set the finished flag, something went wrong.
        if !this.state.contains(WriteStateFlags::FINISHED_WRITE) {
            Poll::Ready(Err(Error::Status(tonic::Status::data_loss(
                "write failed successfully with bytes remaining?",
            ))))
        } else {
            Poll::Ready(extract_object(response))
        }
    }
}
