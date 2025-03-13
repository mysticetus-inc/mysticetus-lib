use std::fmt;
use std::future::Future;
use std::num::{NonZeroI64, NonZeroU64};
use std::ops::{Bound, RangeBounds};
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{BufMut, Bytes};
use futures::Stream;
use num_traits::PrimInt;
use protos::storage::{
    ChecksummedData, CommonObjectRequestParams, ContentRange, Object, ObjectChecksums,
    ReadObjectRequest, ReadObjectResponse,
};

use crate::bucket::BucketClient;
use crate::error::DataError;
use crate::generation::{
    Generation, GenerationPredicate, IfGenerationMatches, IfGenerationNotMatches,
    IfMetaGenerationMatches, IfMetaGenerationNotMatches,
};
use crate::util::OwnedOrMut;

pub struct ReadBuilder<'a, S, GenPredicate, MetaGenPredicate> {
    client: OwnedOrMut<'a, BucketClient>,
    path: S,
    read_range: ReadRange,
    gen_pred: GenPredicate,
    meta_gen_pred: MetaGenPredicate,
    common_request_params: Option<CommonObjectRequestParams>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadRange {
    // Allowed to be negative, to indicate reading the last N bytes of the object
    // (i.e reading with an offset -5 would return the final 5 bytes)
    offset: Option<NonZeroI64>,
    // 0 means no limit, as per the ReadObjectRequest.read_limit docs.
    // The stream can return fewer bytes if the object isn't large enough.
    //
    // A negative value here is invalid.
    limit: Option<NonZeroI64>,
}

impl Default for ReadRange {
    #[inline]
    fn default() -> Self {
        Self::all()
    }
}

impl ReadRange {
    fn from_range<I: PrimInt>(range: impl RangeBounds<I>) -> Result<Self, InvalidReadBounds> {
        enum NormalizedBound {
            Zero,
            NonZero(NonZeroI64),
            Unbounded,
        }

        impl From<i64> for NormalizedBound {
            fn from(value: i64) -> Self {
                match NonZeroI64::new(value) {
                    Some(nz) => Self::NonZero(nz),
                    None => Self::Zero,
                }
            }
        }

        // converts + normalizes range bounds to be
        // { start: Inclusive | Unbounded, end: Exclusive | Unbounded }
        fn normalize_bounds<I: PrimInt>(
            start: Bound<&I>,
            end: Bound<&I>,
        ) -> Result<(NormalizedBound, NormalizedBound), InvalidReadBounds> {
            fn convert_and_add_one(
                int: &impl PrimInt,
            ) -> Result<NormalizedBound, InvalidReadBounds> {
                let int_i64 = to_i64(int)?;
                let int = int_i64
                    .checked_add(1)
                    .ok_or_else(|| InvalidReadBounds::out_of_range(*int))?;
                Ok(NormalizedBound::from(int))
            }

            let start = match start {
                Bound::Unbounded => NormalizedBound::Unbounded,
                Bound::Included(incl) => NormalizedBound::from(to_i64(incl)?),
                Bound::Excluded(excl) => convert_and_add_one(excl)?,
            };

            let end = match end {
                Bound::Unbounded => NormalizedBound::Unbounded,
                Bound::Excluded(excl) => NormalizedBound::from(to_i64(excl)?),
                Bound::Included(incl) => convert_and_add_one(incl)?,
            };

            Ok((start, end))
        }

        fn inner(
            start: NormalizedBound,
            end: NormalizedBound,
        ) -> Result<ReadRange, InvalidReadBounds> {
            use NormalizedBound::*;

            match (start, end) {
                (Unbounded | Zero, Unbounded) => Ok(ReadRange::all()),
                (Zero, Zero) => Err(InvalidReadBounds::ZeroByteLimit),
                (Unbounded | Zero, NonZero(end)) if end.is_negative() => {
                    Err(InvalidReadBounds::NegativeEndBoundWithoutStartBound)
                }
                (Unbounded | Zero, NonZero(end)) => Ok(ReadRange {
                    limit: Some(end),
                    offset: None,
                }),
                (NonZero(start), Unbounded | Zero) => Ok(ReadRange {
                    offset: Some(start),
                    limit: None,
                }),
                (Unbounded, Zero) => Err(InvalidReadBounds::ZeroByteLimit),
                (NonZero(start), NonZero(end)) => {
                    let limit = end.get() - start.get();

                    match NonZeroI64::new(limit) {
                        Some(limit) if limit.is_positive() => Ok(ReadRange {
                            offset: Some(start),
                            limit: Some(limit),
                        }),
                        Some(limit) => Err(InvalidReadBounds::NegativeLimit(limit)),
                        None => Err(InvalidReadBounds::ZeroByteLimit),
                    }
                }
            }
        }

        let (start, end) = normalize_bounds(range.start_bound(), range.end_bound())?;

        inner(start, end)
    }

    #[inline]
    pub const fn all() -> Self {
        Self {
            limit: None,
            offset: None,
        }
    }
}

impl<'a, S> ReadBuilder<'a, S, (), ()> {
    pub(crate) fn new(client: impl Into<OwnedOrMut<'a, BucketClient>>, path: S) -> Self {
        Self {
            client: client.into(),
            path,
            read_range: ReadRange::default(),
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

    pub fn into_static(self) -> ReadBuilder<'static, S, GenPred, MetaGenPred> {
        ReadBuilder {
            path: self.path,
            read_range: self.read_range,
            gen_pred: self.gen_pred,
            meta_gen_pred: self.meta_gen_pred,
            client: self.client.into_static(),
            common_request_params: self.common_request_params,
        }
    }

    pub fn range<Int: PrimInt>(
        mut self,
        range: impl RangeBounds<Int>,
    ) -> Result<Self, InvalidReadBounds> {
        self.read_range = ReadRange::from_range(range)?;
        Ok(self)
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
            read_range: self.read_range,
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
            read_range: self.read_range,
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

impl<'a, S, GenPredicate, MetaGenPredicate> ReadBuilder<'a, S, GenPredicate, MetaGenPredicate>
where
    S: Into<String>,
    GenPredicate: GenerationPredicate<ReadObjectRequest>,
    MetaGenPredicate: GenerationPredicate<ReadObjectRequest>,
{
    fn read_inner(
        self,
    ) -> impl Future<Output = tonic::Result<tonic::Response<tonic::Streaming<ReadObjectResponse>>>>
    + Send
    + 'a {
        let mut request = ReadObjectRequest {
            bucket: self.client.qualified_bucket().to_owned(),
            object: self.path.into(),
            common_object_request_params: self.common_request_params,
            read_limit: self.read_range.limit.map(NonZeroI64::get).unwrap_or(0),
            read_offset: self.read_range.offset.map(NonZeroI64::get).unwrap_or(0),
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

    pub fn stream(self) -> impl Future<Output = crate::Result<ReadStream>> + Send + 'a {
        let fut = self.read_inner();

        async move {
            let response = fut.await?;
            ReadStream::new(response.into_inner()).await
        }
    }
}

/*
// TODO: uncomment when 'impl_trait_in_assoc_type' stabilizes.
// see: https://github.com/rust-lang/rust/issues/63063
impl<S, GenPredicate, MetaGenPredicate> std::future::IntoFuture
    for ReadBuilder<'_, S, GenPredicate, MetaGenPredicate>
where
    S: Into<String>,
    GenPredicate: GenerationPredicate<ReadObjectRequest>,
    MetaGenPredicate: GenerationPredicate<ReadObjectRequest>,
{
    type Output = crate::Result<ReadStream>;
    type IntoFuture = impl Future<Output = crate::Result<ReadStream>> + Send + 'static;

    fn into_future(self) -> Self::IntoFuture {
        self.stream()
    }
}
*/

crate::generation::impl_generation_predicate! {
    |self: Generation, req: ReadObjectRequest| req.generation = self.0 as i64,
    |self: IfGenerationMatches, req: ReadObjectRequest| req.if_generation_match = Some(self.0 as i64),
    |self: IfGenerationNotMatches, req: ReadObjectRequest| req.if_generation_not_match = Some(self.0 as i64),
    |self: IfMetaGenerationMatches, req: ReadObjectRequest| req.if_metageneration_match = Some(self.0 as i64),
    |self: IfMetaGenerationNotMatches, req: ReadObjectRequest| req.if_metageneration_not_match = Some(self.0 as i64),
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidReadBounds {
    #[error("negative read limit of {0} is invalid")]
    NegativeLimit(NonZeroI64),
    #[error("bounds of {start} and {end} are invalid")]
    InvalidBounds { start: i64, end: i64 },
    #[error("offset of {0} is invalid, out of range for an i64")]
    OutOfRange(NonZeroU64),
    #[error("invalid 0 byte limit")]
    ZeroByteLimit,
    #[error("can't compute a limit with a negative end bound and no start bound")]
    NegativeEndBoundWithoutStartBound,
}

impl InvalidReadBounds {
    fn out_of_range(invalid: impl PrimInt) -> Self {
        let int = invalid.to_u64().expect("unrepresentable by a u64");
        Self::OutOfRange(NonZeroU64::new(int).expect("InvalidReadBounds::out_of_range called on 0"))
    }
}

fn to_i64(int: &impl PrimInt) -> Result<i64, InvalidReadBounds> {
    if let Some(i) = int.to_i64() {
        return Ok(i);
    }

    Err(InvalidReadBounds::out_of_range(*int))
}

impl From<std::convert::Infallible> for InvalidReadBounds {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

enum Validator {
    Crc32c {
        current_crc: u32,
        expected_crc: u32,
    },
    Md5 {
        expected_md5: md5::Digest,
        hasher: md5::Context,
    },
}

impl Validator {
    fn new(checksums: ObjectChecksums) -> Option<Self> {
        fn md5_to_array(md5: Bytes) -> Option<md5::Digest> {
            md5.as_ref().try_into().ok().map(md5::Digest)
        }

        match checksums.crc32c {
            Some(expected_crc) => Some(Self::Crc32c {
                current_crc: 0,
                expected_crc,
            }),
            None => md5_to_array(checksums.md5_hash).map(|expected_md5| Self::Md5 {
                hasher: md5::Context::new(),
                expected_md5,
            }),
        }
    }

    fn update(&mut self, bytes: &[u8], computed_crc: Option<u32>) {
        match self {
            Self::Crc32c { current_crc, .. } => {
                let computed_crc = computed_crc.unwrap_or_else(|| crc32c::crc32c(bytes));
                *current_crc = crc32c::crc32c_combine(*current_crc, computed_crc, bytes.len());
            }
            Self::Md5 { hasher, .. } => hasher.consume(bytes),
        }
    }

    fn finish(&mut self) -> crate::Result<()> {
        match self {
            Self::Crc32c {
                current_crc,
                expected_crc,
            } => {
                if *current_crc != *expected_crc {
                    Err(crate::Error::DataError(DataError::crc32c(
                        *expected_crc,
                        *current_crc,
                    )))
                } else {
                    Ok(())
                }
            }
            Self::Md5 {
                expected_md5,
                hasher,
            } => {
                let digest = std::mem::replace(hasher, md5::Context::new()).compute();
                if &digest.0[..] != &expected_md5[..] {
                    Err(crate::Error::DataError(DataError::md5(
                        *expected_md5,
                        digest,
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }
}

pin_project_lite::pin_project! {
    pub struct ReadStream {
        pub metadata: Object,
        content_range: Option<ContentRange>,
        validator: Option<Validator>,
        leading_chunk: Option<ChecksummedData>,
        #[pin]
        stream: tonic::Streaming<ReadObjectResponse>,
    }
}

impl ReadStream {
    pub(crate) async fn new(
        mut stream: tonic::Streaming<ReadObjectResponse>,
    ) -> crate::Result<Self> {
        let mut pinned = Pin::new(&mut stream);
        let mut validator = None;

        let first_message =
            match std::future::poll_fn(|cx| poll_stream(&mut pinned, &mut validator, cx)).await {
                Some(result) => result?,
                None => {
                    return Err(crate::Error::internal(
                        "ReadObject never returned a response",
                    ));
                }
            };

        let ReadObjectResponse {
            checksummed_data,
            object_checksums,
            metadata,
            content_range,
        } = first_message;

        let metadata = metadata.ok_or_else(|| {
            crate::Error::internal("First ReadObject response missing expected metadata")
        })?;

        let validator = match content_range {
            Some(ref range) if range.end - range.start == metadata.size => {
                object_checksums.and_then(Validator::new)
            }
            Some(_) => None,
            None => object_checksums.and_then(Validator::new),
        };

        Ok(Self {
            metadata,
            validator,
            leading_chunk: checksummed_data,
            content_range,
            stream,
        })
    }

    pub fn content_range(&self) -> Option<&ContentRange> {
        self.content_range.as_ref()
    }

    pub async fn collect_to_vec(self) -> crate::Result<(Object, Vec<u8>)> {
        let len = self
            .content_range
            .as_ref()
            .map(|range| range.end - range.start)
            .unwrap_or(self.metadata.size) as usize;

        let mut vec = Vec::with_capacity(len);

        let object = self.collect_into(&mut vec).await?;

        Ok((object, vec))
    }

    pub async fn collect_into<B: BufMut + ?Sized>(
        mut self,
        mut buf: &mut B,
    ) -> crate::Result<Object> {
        while let Some(result) = std::future::poll_fn(|cx| Pin::new(&mut self).poll_next(cx)).await
        {
            let chunk = result?;
            (&mut buf).put(chunk);
        }

        Ok(self.metadata)
    }
}

impl fmt::Debug for ReadStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadStream")
            .field("content_range", &self.content_range)
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

impl Stream for ReadStream {
    type Item = crate::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if let Some(data) = this
            .leading_chunk
            .take()
            .and_then(|data| handle_data(data, this.validator))
        {
            return Poll::Ready(Some(data));
        }

        loop {
            let resp = match std::task::ready!(this.stream.as_mut().poll_next(cx)) {
                None => {
                    if let Some(validator) = this.validator {
                        validator.finish()?;
                    }
                    return Poll::Ready(None);
                }
                Some(result) => result?,
            };

            if let Some(content_range) = resp.content_range {
                *this.content_range = Some(content_range);
            }

            if this.validator.is_none() {
                if let Some(validator) = resp.object_checksums.and_then(Validator::new) {
                    *this.validator = Some(validator);
                }
            }

            if let Some(result) = resp
                .checksummed_data
                .and_then(|data| handle_data(data, this.validator))
            {
                return Poll::Ready(Some(result));
            }
        }
    }
}

fn handle_data(
    data: ChecksummedData,
    validator: &mut Option<Validator>,
) -> Option<crate::Result<Bytes>> {
    if data.content.is_empty() {
        return None;
    }

    if let Some(crc) = data.crc32c {
        let computed = crc32c::crc32c(&data.content);

        if crc != computed {
            return Some(Err(crate::Error::DataError(DataError::crc32c(
                crc, computed,
            ))));
        }

        if let Some(validator) = validator {
            validator.update(&data.content, Some(computed));
        }
    } else if let Some(validator) = validator {
        validator.update(&data.content, None);
    }

    Some(Ok(data.content))
}

fn poll_stream(
    stream: &mut Pin<&mut tonic::Streaming<ReadObjectResponse>>,
    validator: &mut Option<Validator>,
    cx: &mut Context<'_>,
) -> Poll<Option<crate::Result<ReadObjectResponse>>> {
    match std::task::ready!(stream.as_mut().poll_next(cx)) {
        None => {
            if let Some(validator) = validator {
                validator.finish()?;
            }
            Poll::Ready(None)
        }
        Some(result) => Poll::Ready(Some(result.map_err(crate::Error::Status))),
    }
}
