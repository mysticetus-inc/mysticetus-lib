use std::future::Future;
use std::num::NonZeroI64;
use std::ops::{Bound, RangeBounds};

use num_traits::PrimInt;
use protos::storage::{CommonObjectRequestParams, ReadObjectRequest, ReadObjectResponse};

use crate::bucket::BucketClient;
use crate::generation::{
    Generation, GenerationPredicate, IfGenerationMatches, IfGenerationNotMatches,
    IfMetaGenerationMatches, IfMetaGenerationNotMatches,
};

pub struct ReadBuilder<'a, S, GenPredicate, MetaGenPredicate> {
    client: &'a BucketClient,
    path: S,
    read_offset: Option<NonZeroI64>,
    // 0 means no limit, as per the ReadObjectRequest.read_limit docs
    read_limit: u64,
    gen_pred: GenPredicate,
    meta_gen_pred: MetaGenPredicate,
    common_request_params: Option<CommonObjectRequestParams>,
}

impl<'a, S> ReadBuilder<'a, S, (), ()> {
    pub(crate) fn new(client: &'a BucketClient, path: S) -> Self {
        Self {
            client,
            path,
            read_limit: 0,
            read_offset: None,
            gen_pred: (),
            meta_gen_pred: (),
            common_request_params: None,
        }
    }
}

impl<'a, S, GenPred, MetaGenPred> ReadBuilder<'a, S, GenPred, MetaGenPred> {
    pub fn common_object_request_params(mut self, params: CommonObjectRequestParams) -> Self {
        self.common_request_params = Some(params);
        self
    }

    pub fn read_limit(mut self, read_limit: u64) -> Self {
        self.read_limit = read_limit;
        self
    }

    pub fn read_offset(mut self, read_offset: i64) -> Self {
        self.read_offset = NonZeroI64::new(read_offset);
        self
    }

    /// Translates a range bound into separate calls to [`Self::read_limit`] and
    /// [`Self::read_offset`]
    pub fn range<R, Int>(self, range: R) -> Self
    where
        R: RangeBounds<Int>,
        Int: PrimInt + num_traits::ToPrimitive,
    {
        let start = match range.start_bound() {
            Bound::Unbounded => None,
            Bound::Included(start) => Some(*start),
            Bound::Excluded(excl) => Some(
                excl.checked_add(&Int::one())
                    .expect("exclusionary start bound overflow"),
            ),
        };

        let end = match range.end_bound() {
            Bound::Unbounded => None,
            Bound::Included(start) => Some(
                start
                    .checked_sub(&Int::one())
                    .expect("included end bound overflow"),
            ),
            Bound::Excluded(excl) => Some(*excl),
        };

        match (start, end) {
            (None, None) => self,
            (Some(start), None) => {
                self.read_offset(start.to_i64().expect("start out of range for i64"))
            }
            (None, Some(end)) => self.read_limit(end.to_u64().expect("end out of range for u64")),
            (Some(start), Some(end)) => {
                let delta = end
                    .checked_sub(&start)
                    .expect("getting delta from start and end overflows");

                self.read_offset(start.to_i64().expect("start out of range for i64"))
                    .read_limit(delta.to_u64().expect("delta out of range for u64"))
            }
        }
    }
}

impl<'a, S, MetaGenPred> ReadBuilder<'a, S, (), MetaGenPred> {
    fn with_generation_predicate<GenPred>(
        self,
        gen_pred: GenPred,
    ) -> ReadBuilder<'a, S, GenPred, MetaGenPred> {
        ReadBuilder {
            client: self.client,
            path: self.path,
            read_limit: self.read_limit,
            read_offset: self.read_offset,
            common_request_params: self.common_request_params,
            gen_pred,
            meta_gen_pred: self.meta_gen_pred,
        }
    }

    pub fn generation(self, generation: u64) -> ReadBuilder<'a, S, Generation, MetaGenPred> {
        self.with_generation_predicate(Generation(generation))
    }

    pub fn if_generation_matches(
        self,
        generation: u64,
    ) -> ReadBuilder<'a, S, IfGenerationMatches, MetaGenPred> {
        self.with_generation_predicate(IfGenerationMatches(generation))
    }

    pub fn if_generation_not_matches(
        self,
        generation: u64,
    ) -> ReadBuilder<'a, S, IfGenerationNotMatches, MetaGenPred> {
        self.with_generation_predicate(IfGenerationNotMatches(generation))
    }
}

impl<'a, S, GenPred> ReadBuilder<'a, S, GenPred, ()> {
    fn with_meta_generation_predicate<MetaGenPred>(
        self,
        meta_gen_pred: MetaGenPred,
    ) -> ReadBuilder<'a, S, GenPred, MetaGenPred> {
        ReadBuilder {
            client: self.client,
            path: self.path,
            read_limit: self.read_limit,
            read_offset: self.read_offset,
            common_request_params: self.common_request_params,
            gen_pred: self.gen_pred,
            meta_gen_pred,
        }
    }

    pub fn if_meta_generation_matches(
        self,
        meta_generation: u64,
    ) -> ReadBuilder<'a, S, GenPred, IfMetaGenerationMatches> {
        self.with_meta_generation_predicate(IfMetaGenerationMatches(meta_generation))
    }

    pub fn if_meta_generation_not_matches(
        self,
        meta_generation: u64,
    ) -> ReadBuilder<'a, S, GenPred, IfMetaGenerationNotMatches> {
        self.with_meta_generation_predicate(IfMetaGenerationNotMatches(meta_generation))
    }
}

impl<S, GenPredicate, MetaGenPredicate> ReadBuilder<'_, S, GenPredicate, MetaGenPredicate>
where
    S: Into<String>,
    GenPredicate: GenerationPredicate<ReadObjectRequest>,
    MetaGenPredicate: GenerationPredicate<ReadObjectRequest>,
{
    fn read_inner(
        self,
    ) -> impl Future<Output = tonic::Result<tonic::Response<tonic::Streaming<ReadObjectResponse>>>>
    + Send
    + 'static {
        let mut request = ReadObjectRequest {
            bucket: self.client.qualified_bucket().to_owned(),
            object: self.path.into(),
            common_object_request_params: self.common_request_params,
            read_limit: self.read_limit as i64,
            read_offset: self.read_offset.map(NonZeroI64::get).unwrap_or(0),
            generation: 0,
            read_mask: None,
            if_generation_match: None,
            if_generation_not_match: None,
            if_metageneration_match: None,
            if_metageneration_not_match: None,
        };

        self.gen_pred.insert(&mut request);
        self.meta_gen_pred.insert(&mut request);

        let mut client = self.client.client();

        async move { client.read_object(request).await }
    }
}

crate::generation::impl_generation_predicate! {
    |self: Generation, req: ReadObjectRequest| req.generation = self.0 as i64,
    |self: IfGenerationMatches, req: ReadObjectRequest| req.if_generation_match = Some(self.0 as i64),
    |self: IfGenerationNotMatches, req: ReadObjectRequest| req.if_generation_not_match = Some(self.0 as i64),
    |self: IfMetaGenerationMatches, req: ReadObjectRequest| req.if_metageneration_match = Some(self.0 as i64),
    |self: IfMetaGenerationNotMatches, req: ReadObjectRequest| req.if_metageneration_not_match = Some(self.0 as i64),
}
