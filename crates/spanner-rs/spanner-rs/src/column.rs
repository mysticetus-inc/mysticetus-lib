use crate::convert::SpannerEncode;
use crate::queryable::Queryable;
use crate::ty::SpannerType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Column<'a> {
    pub index: usize,
    pub name: &'a str,
    pub ty: &'a crate::ty::Type,
    pub nullable: bool,
}

impl Column<'static> {
    pub const fn new<T: SpannerEncode>(index: usize, name: &'static str) -> Self {
        Self {
            index,
            name,
            ty: &<T::SpannerType as SpannerType>::TYPE,
            nullable: <T::SpannerType as SpannerType>::NULLABLE,
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("invalid column index {index}, row contains {len} columns")]
pub struct InvalidColumnIndex {
    index: usize,
    len: usize,
}

impl InvalidColumnIndex {
    pub(crate) const fn new_explicit(index: usize, len: usize) -> Self {
        Self { index, len }
    }

    #[allow(unused)]
    pub(crate) const fn new<T: Queryable>(index: usize) -> Self {
        Self::new_explicit(index, <T::NumColumns as typenum::Unsigned>::USIZE)
    }
}

/*
#[doc(hidden)]
pub mod infer {
    use std::marker::PhantomData;

    use super::Column;
    use crate::convert::SpannerEncode;
    use crate::ty::SpannerType;

    #[doc(hidden)]
    pub struct Nullable<const NULLABLE: bool>;

    #[const_trait]
    pub trait InferNonNullable {
        fn infer(&self) -> Nullable<false> {
            Nullable
        }
    }

    #[const_trait]
    pub trait InferNullable {
        fn infer(&self) -> Nullable<true> {
            Nullable
        }
    }


    impl<const NULLABLE: bool> Nullable<NULLABLE> {
        pub const fn build<T: SpannerEncode>(self, index: usize, name: &'static str) -> Column {
            Column {
                index,
                name,
                ty: <T::SpannerType as SpannerType>::TYPE,
                nullable: NULLABLE,
            }
        }
    }

    impl<T: SpannerEncode> const InferNonNullable for &PhantomData<T> { }

    impl<T: SpannerEncode> const InferNullable for PhantomData<Option<T>> { }
}
*/
