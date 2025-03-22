use typenum::{Bit, False, True};

use super::Type;
use super::markers::{self, SpannerTypeSealed};

/// Implemented by types that have a spanner encoded counterpart.
pub trait SpannerType {
    /// The underlying spanner type.
    type Type: SpannerTypeSealed;

    /// Whether a type is nullable or not.
    type Nullable: Bit;
}

pub(crate) const fn ty<T: SpannerType + ?Sized>() -> &'static Type {
    <T::Type as SpannerTypeSealed>::TYPE
}

pub(crate) const fn nullable<T: SpannerType + ?Sized>() -> bool {
    <T::Nullable as Bit>::BOOL
}

impl<T: SpannerType + ?Sized> SpannerType for &T {
    type Type = T::Type;
    type Nullable = T::Nullable;
}

impl<T: SpannerType + ?Sized> SpannerType for &mut T {
    type Type = T::Type;
    type Nullable = T::Nullable;
}

macro_rules! impl_scalar {
    ($(
        $(#[$cfg_attr:meta])?
        $name:ty => $t:ident
    ),* $(,)?) => {
        $(
            $(#[$cfg_attr])?
            impl SpannerType for $name {
                type Type = markers::$t;
                type Nullable = False;
            }
        )*
    };
}

impl_scalar! {
    isize => Int64,
    i128 => Numeric,
    i64 => Int64,
    i32 => Int64,
    i16 => Int64,
    i8 => Int64,
    usize => Int64,
    u128 => Numeric,
    u64 => Int64,
    u32 => Int64,
    u16 => Int64,
    u8 => Int64,
    f64 => Float64,
    f32 => Float64,
    bool => Bool,
    timestamp::Timestamp => Timestamp,
    timestamp::Date => Date,
    str => String,
    String => String,
    bytes::Bytes => Bytes,
    [u8] => Bytes,
    Vec<u8> => Bytes,
    #[cfg(feature = "serde_json")]
    serde_json::Value => Json,
}

macro_rules! spanner_type_defer_to {
    ($($parent:ty: $deferred:ty),* $(,)?) => {
        $(
            impl $crate::ty::SpannerType for $parent {
                type Type = <$deferred as $crate::ty::SpannerType>::Type;

                type Nullable = <$deferred as $crate::ty::SpannerType>::Nullable;
            }
        )*
    };
}

spanner_type_defer_to! {
    std::num::NonZeroU8: u8,
    std::num::NonZeroU16: u16,
    std::num::NonZeroU32: u32,
    std::num::NonZeroU64: u64,
    std::num::NonZeroU128: u128,
    std::num::NonZeroUsize: usize,
    std::num::NonZeroI8: i8,
    std::num::NonZeroI16: i16,
    std::num::NonZeroI32: i32,
    std::num::NonZeroI64: i64,
    std::num::NonZeroI128: i128,
    std::num::NonZeroIsize: isize,
}

impl<T: SpannerType> SpannerType for Option<T> {
    type Type = T::Type;
    type Nullable = True;
}

macro_rules! impl_for_wrapper_type {
    ($($t:ty),* $(,)?) => {
        $(
            impl<T: SpannerType + ?Sized> SpannerType for $t {
                type Type = T::Type;
                type Nullable = T::Nullable;
            }
        )*
    };
}

impl_for_wrapper_type! {
    Box<T>,
    std::sync::Arc<T>,
    std::rc::Rc<T>,
}

impl<T> SpannerType for std::borrow::Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: SpannerType,
{
    type Type = <T::Owned as SpannerType>::Type;
    type Nullable = <T::Owned as SpannerType>::Nullable;
}
