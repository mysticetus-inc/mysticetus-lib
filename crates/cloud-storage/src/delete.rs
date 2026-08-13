use protos::storage::{CommonObjectRequestParams, DeleteObjectRequest};

use crate::util::OwnedOrMut;

pub struct DeleteBuilder<'a> {
    client: OwnedOrMut<'a, crate::BucketClient>,
    request: DeleteObjectRequest,
}

impl<'a> DeleteBuilder<'a> {
    #[inline]
    pub(crate) fn new(
        client: impl Into<OwnedOrMut<'a, crate::BucketClient>>,
        path: impl Into<String>,
    ) -> Self {
        let client = client.into();

        let request = DeleteObjectRequest {
            bucket: client.qualified_bucket().to_owned(),
            object: path.into(),
            generation: 0,
            if_generation_match: None,
            if_generation_not_match: None,
            if_metageneration_match: None,
            if_metageneration_not_match: None,
            common_object_request_params: None,
        };

        Self { client, request }
    }

    #[inline]
    pub fn into_static(self) -> DeleteBuilder<'static> {
        let Self { client, request } = self;
        let client = client.into_static();
        DeleteBuilder { client, request }
    }

    #[inline]
    pub fn common_object_request_params(
        mut self,
        request_params: CommonObjectRequestParams,
    ) -> Self {
        self.request.common_object_request_params = Some(request_params);
        self
    }

    #[inline]
    pub fn generation(mut self, generation: i64) -> Self {
        self.request.generation = generation;
        self
    }

    #[inline]
    pub fn if_generation_matches(mut self, generation: i64) -> Self {
        self.request.if_generation_match = Some(generation);
        self
    }

    #[inline]
    pub fn if_generation_not_matches(mut self, generation: i64) -> Self {
        self.request.if_generation_not_match = Some(generation);
        self
    }

    #[inline]
    pub fn if_metageneration_matches(mut self, metageneration: i64) -> Self {
        self.request.if_metageneration_match = Some(metageneration);
        self
    }

    #[inline]
    pub fn if_metageneration_not_matches(mut self, metageneration: i64) -> Self {
        self.request.if_metageneration_not_match = Some(metageneration);
        self
    }

    #[inline]
    pub async fn delete(self) -> crate::Result<()> {
        let Self {
            mut client,
            request,
        } = self;

        client.client_mut().delete_object(request).await?;

        Ok(())
    }
}

impl<'a> IntoFuture for DeleteBuilder<'a> {
    type IntoFuture = std::pin::Pin<Box<dyn Future<Output = crate::Result<()>> + Send + 'a>>;
    type Output = crate::Result<()>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.delete())
    }
}
