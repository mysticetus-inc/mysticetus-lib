use std::future::Future;

use protos::storage::{ListObjectsRequest, ListObjectsResponse};

use crate::bucket::BucketClient;

pub const MAX_PAGE_SIZE: u32 = 1000;

pub struct ListBuilder<'a, S> {
    client: &'a BucketClient,
    page_size: u32,
    prefix: Option<String>,
    state: S,
}

pub trait ListState {
    fn complete_response(self, client: &BucketClient, prefix: Option<String>)
    -> ListObjectsRequest;
}

impl<'a> ListBuilder<'a, ()> {
    pub(crate) fn new(client: &'a BucketClient) -> Self {
        Self {
            client,
            page_size: MAX_PAGE_SIZE,
            prefix: None,
            state: (),
        }
    }
}

impl<S: ListState> ListBuilder<'_, S> {
    fn list_inner(
        self,
    ) -> impl Future<Output = tonic::Result<(BucketClient, tonic::Response<ListObjectsResponse>)>>
    + Send
    + 'static {
        let Self {
            client,
            page_size,
            prefix,
            state,
        } = self;

        let mut request = state.complete_response(client, prefix);
        let page_size = page_size.min(MAX_PAGE_SIZE as u32) as i32;

        request.page_size = page_size;

        let mut client = self.client.clone();

        async move {
            let resp = client.client_mut().list_objects(request).await?;
            Ok((client, resp))
        }
    }
}
