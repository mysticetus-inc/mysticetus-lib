use std::future::Future;

use protos::storage::{CommonObjectRequestParams, GetObjectRequest, Object};

use crate::bucket::BucketClient;
use crate::generation::{
    Generation, GenerationPredicate, IfGenerationMatches, IfGenerationNotMatches,
    IfMetaGenerationMatches, IfMetaGenerationNotMatches,
};

pub struct GetBuilder<'a, S, GenPredicate, MetaGenPredicate> {
    client: &'a BucketClient,
    path: S,
    gen_pred: GenPredicate,
    meta_gen_pred: MetaGenPredicate,
    common_request_params: Option<CommonObjectRequestParams>,
}

impl<'a, S> GetBuilder<'a, S, (), ()> {
    pub(crate) fn new(client: &'a BucketClient, path: S) -> Self {
        Self {
            client,
            path,
            gen_pred: (),
            meta_gen_pred: (),
            common_request_params: None,
        }
    }
}

impl<'a, S, GenPred, MetaGenPred> GetBuilder<'a, S, GenPred, MetaGenPred> {
    pub fn common_object_request_params(mut self, params: CommonObjectRequestParams) -> Self {
        self.common_request_params = Some(params);
        self
    }
}

impl<'a, S, MetaGenPred> GetBuilder<'a, S, (), MetaGenPred> {
    fn with_generation_predicate<GenPred>(
        self,
        gen_pred: GenPred,
    ) -> GetBuilder<'a, S, GenPred, MetaGenPred> {
        GetBuilder {
            client: self.client,
            path: self.path,
            common_request_params: self.common_request_params,
            gen_pred,
            meta_gen_pred: self.meta_gen_pred,
        }
    }

    pub fn generation(self, generation: u64) -> GetBuilder<'a, S, Generation, MetaGenPred> {
        self.with_generation_predicate(Generation(generation))
    }

    pub fn if_generation_matches(
        self,
        generation: u64,
    ) -> GetBuilder<'a, S, IfGenerationMatches, MetaGenPred> {
        self.with_generation_predicate(IfGenerationMatches(generation))
    }

    pub fn if_generation_not_matches(
        self,
        generation: u64,
    ) -> GetBuilder<'a, S, IfGenerationNotMatches, MetaGenPred> {
        self.with_generation_predicate(IfGenerationNotMatches(generation))
    }
}

impl<'a, S, GenPred> GetBuilder<'a, S, GenPred, ()> {
    fn with_meta_generation_predicate<MetaGenPred>(
        self,
        meta_gen_pred: MetaGenPred,
    ) -> GetBuilder<'a, S, GenPred, MetaGenPred> {
        GetBuilder {
            client: self.client,
            path: self.path,
            common_request_params: self.common_request_params,
            gen_pred: self.gen_pred,
            meta_gen_pred,
        }
    }

    pub fn if_meta_generation_matches(
        self,
        meta_generation: u64,
    ) -> GetBuilder<'a, S, GenPred, IfMetaGenerationMatches> {
        self.with_meta_generation_predicate(IfMetaGenerationMatches(meta_generation))
    }

    pub fn if_meta_generation_not_matches(
        self,
        meta_generation: u64,
    ) -> GetBuilder<'a, S, GenPred, IfMetaGenerationNotMatches> {
        self.with_meta_generation_predicate(IfMetaGenerationNotMatches(meta_generation))
    }
}

impl<S, GenPredicate, MetaGenPredicate> GetBuilder<'_, S, GenPredicate, MetaGenPredicate>
where
    S: Into<String>,
    GenPredicate: GenerationPredicate<GetObjectRequest>,
    MetaGenPredicate: GenerationPredicate<GetObjectRequest>,
{
    pub fn get(self) -> impl Future<Output = crate::Result<Object>> + Send + 'static {
        let fut = self.get_inner();
        async move {
            fut.await
                .map(tonic::Response::into_inner)
                .map_err(crate::Error::Status)
        }
    }

    pub fn try_get(self) -> impl Future<Output = crate::Result<Option<Object>>> + Send + 'static {
        let fut = self.get_inner();
        async move {
            match fut.await {
                Ok(resp) => Ok(Some(resp.into_inner())),
                Err(err) if matches!(err.code(), tonic::Code::NotFound) => Ok(None),
                Err(err) => Err(crate::Error::Status(err)),
            }
        }
    }

    // use `fn() -> impl Future` instead of `async fn` so we can specify
    // that the returned `Future` is `Send + 'static`
    fn get_inner(
        self,
    ) -> impl Future<Output = tonic::Result<tonic::Response<Object>>> + Send + 'static {
        let mut request = GetObjectRequest {
            bucket: self.client.qualified_bucket().to_owned(),
            object: self.path.into(),
            common_object_request_params: self.common_request_params,
            generation: 0,
            soft_deleted: None,
            restore_token: String::new(),
            read_mask: None,
            if_generation_match: None,
            if_generation_not_match: None,
            if_metageneration_match: None,
            if_metageneration_not_match: None,
        };

        self.gen_pred.insert(&mut request);
        self.meta_gen_pred.insert(&mut request);

        let mut client = self.client.client();

        async move { client.get_object(request).await }
    }
}

crate::generation::impl_generation_predicate! {
    // GetObjectRequest
    |self: Generation, req: GetObjectRequest| req.generation = self.0 as i64,
    |self: IfGenerationMatches, req: GetObjectRequest| req.if_generation_match = Some(self.0 as i64),
    |self: IfGenerationNotMatches, req: GetObjectRequest| req.if_generation_not_match = Some(self.0 as i64),
    |self: IfMetaGenerationMatches, req: GetObjectRequest| req.if_metageneration_match = Some(self.0 as i64),
    |self: IfMetaGenerationNotMatches, req: GetObjectRequest| req.if_metageneration_not_match = Some(self.0 as i64),
}
