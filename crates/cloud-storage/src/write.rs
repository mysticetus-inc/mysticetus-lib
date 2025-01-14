use std::marker::PhantomData;

use protos::storage::write_object_request::FirstMessage;
use protos::storage::{CommonObjectRequestParams, Object, WriteObjectRequest, WriteObjectSpec};

use crate::bucket::BucketClient;

pub struct WriteBuilder<'a, Kind = NonResumable> {
    client: &'a BucketClient,
    common_object_request_params: Option<CommonObjectRequestParams>,
    spec: WriteObjectSpec,
    kind: Kind,
}

pub struct Resumable {}
pub struct NonResumable {}

impl<'a, Kind> WriteBuilder<'a, Kind> {
    pub(crate) fn new<S: Into<String>>(client: &'a BucketClient, path: S) -> Self {
        Self {
            client,
            spec: WriteObjectSpec {
                resource: Some(Object {
                    name: path.into(),
                    ..Default::default()
                }),
                predefined_acl: (),
                if_generation_match: (),
                if_generation_not_match: (),
                if_metageneration_match: (),
                if_metageneration_not_match: (),
                object_size: (),
            },
            common_object_request_params: None,
            kind: todo!(),
        }
    }
}

impl<'a> WriteBuilder<'a, NonResumable> {
    pub(crate) async fn test(self) -> crate::Result<()> {
        let Self {
            spec,
            common_object_request_params,
            client,
            kind,
        } = self;

        let request = WriteObjectRequest {
            write_offset: 0,
            finish_write: false,
            object_checksums: None,
            common_object_request_params,
            first_message: Some(FirstMessage::WriteObjectSpec(spec)),
            data: None,
        };

        todo!();

        let write_status = client.client().write_object(request).await?.into_inner();

        Ok(())
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
