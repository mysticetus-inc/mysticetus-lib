use std::marker::PhantomData;

pub(super) use private::SpannerTypeSealed;

use super::{Field, Type};

macro_rules! impl_primitive_spanner_marker_types {
    ($($name:ident = $t:expr),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum $name {}

            impl private::SpannerTypeSealed for $name {
                const TYPE: &'static Type = $t;
            }
        )*
    };
}

impl_primitive_spanner_marker_types! {
    Int64 = &Type::INT64,
    String = &Type::STRING,
    Float64 = &Type::FLOAT64,
    Numeric = &Type::NUMERIC,
    Bool = &Type::BOOL,
    Timestamp = &Type::TIMESTAMP,
    Date = &Type::DATE,
    Interval = &Type::INTERVAL,
    Bytes = &Type::BYTES,
    Json = &Type::JSON,
}

// --- Array impl -----

#[non_exhaustive]
pub enum Array<T: SpannerTypeSealed + ?Sized> {
    #[doc(hidden)]
    __Type(PhantomData<fn(&T)>),
}

impl<T: SpannerTypeSealed + ?Sized> SpannerTypeSealed for Array<T> {
    const TYPE: &'static Type = &Type::array(T::TYPE);
}

// --- Struct impl -----

pub trait SpannerStruct {
    /// The defined fields for this struct
    const FIELDS: &'static [Field];
}

#[non_exhaustive]
pub enum Struct<T: SpannerStruct + ?Sized> {
    #[doc(hidden)]
    __Type(PhantomData<fn(&T)>),
}

impl<T: SpannerStruct + ?Sized> SpannerTypeSealed for Struct<T> {
    const TYPE: &'static Type = &Type::struct_type(T::FIELDS);
}

// --- Proto trait -----
pub trait SpannerProto {
    const PACKAGE: &'static str;
    const NAME: &'static str;
}

impl<T: prost::Name> SpannerProto for T {
    const NAME: &'static str = <T as prost::Name>::NAME;
    const PACKAGE: &'static str = <T as prost::Name>::PACKAGE;
}

#[non_exhaustive]
pub enum Proto<T: SpannerProto + ?Sized> {
    #[doc(hidden)]
    __Type(PhantomData<fn(&T)>),
}

impl<T: SpannerProto + ?Sized> SpannerTypeSealed for Proto<T> {
    const TYPE: &'static Type = &Type::proto::<T>();
}

// --- Enum trait -----

pub trait SpannerEnum {
    const PACKAGE: &'static str;
    const NAME: &'static str;
}

#[non_exhaustive]
pub enum ProtoEnum<T: SpannerEnum + ?Sized> {
    #[doc(hidden)]
    __Type(PhantomData<fn(&T)>),
}

impl<T: SpannerEnum + ?Sized> SpannerTypeSealed for ProtoEnum<T> {
    const TYPE: &'static Type = &Type::proto_enum::<T>();
}

mod private {
    use crate::Type;

    pub trait SpannerTypeSealed {
        const TYPE: &'static Type;
    }

    impl<T: SpannerTypeSealed + ?Sized> SpannerTypeSealed for &T {
        const TYPE: &'static Type = T::TYPE;
    }

    impl<T: SpannerTypeSealed + ?Sized> SpannerTypeSealed for &mut T {
        const TYPE: &'static Type = T::TYPE;
    }

    impl<T: SpannerTypeSealed + ?Sized> SpannerTypeSealed for Box<T> {
        const TYPE: &'static Type = T::TYPE;
    }

    impl<T: SpannerTypeSealed + ?Sized> SpannerTypeSealed for std::sync::Arc<T> {
        const TYPE: &'static Type = T::TYPE;
    }

    impl<T: SpannerTypeSealed + ?Sized> SpannerTypeSealed for std::rc::Rc<T> {
        const TYPE: &'static Type = T::TYPE;
    }
}
