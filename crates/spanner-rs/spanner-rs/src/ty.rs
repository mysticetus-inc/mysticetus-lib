use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

use bytes::Bytes;
use protos::protobuf::ListValue;
use protos::protobuf::value::Kind;
use protos::spanner::{self, StructType, TypeAnnotationCode, TypeCode, struct_type};
use shared::static_or_boxed::StaticOrBoxed;
use timestamp::{Date, Timestamp};

use crate::error::{ConvertError, FromError, MissingTypeInfo};
use crate::value::EncodedArray;

/// Implemented by types that have a 'native' spanner analog, i.e
/// [`String`] == 'STRING', [`i64`] == 'INT64', etc.
pub trait SpannerType {
    /// The native Spanner [`Type`].
    const TYPE: &'static Type;

    /// Whether a type is nullable or not.
    const NULLABLE: bool;
}

impl<T: SpannerType + ?Sized> SpannerType for &T {
    const TYPE: &'static Type = T::TYPE;

    const NULLABLE: bool = T::NULLABLE;
}

impl<T: SpannerType + ?Sized> SpannerType for &mut T {
    const TYPE: &'static Type = T::TYPE;

    const NULLABLE: bool = T::NULLABLE;
}

macro_rules! impl_scalar {
    ($(
        $(#[$cfg_attr:meta])?
        $name:ty => $t:expr
    ),* $(,)?) => {
        $(
            $(#[$cfg_attr])?
            impl SpannerType for $name {
                const TYPE: &'static Type = &$t;

                const NULLABLE: bool = false;
            }
        )*
    };
}

impl_scalar! {
    isize => Type::Scalar(Scalar::Int64),
    i128 => Type::Scalar(Scalar::Numeric),
    i64 => Type::Scalar(Scalar::Int64),
    i32 => Type::Scalar(Scalar::Int64),
    i16 => Type::Scalar(Scalar::Int64),
    i8 => Type::Scalar(Scalar::Int64),
    usize => Type::Scalar(Scalar::Int64),
    u128 => Type::Scalar(Scalar::Numeric),
    u64 => Type::Scalar(Scalar::Int64),
    u32 => Type::Scalar(Scalar::Int64),
    u16 => Type::Scalar(Scalar::Int64),
    u8 => Type::Scalar(Scalar::Int64),
    f64 => Type::Scalar(Scalar::Float64),
    f32 => Type::Scalar(Scalar::Float64),
    bool => Type::Scalar(Scalar::Bool),
    Timestamp => Type::Scalar(Scalar::Timestamp),
    Date => Type::Scalar(Scalar::Date),
    str => Type::Scalar(Scalar::String),
    String => Type::Scalar(Scalar::String),
    Bytes => Type::Scalar(Scalar::Bytes),
    [u8] => Type::Scalar(Scalar::Bytes),
    Vec<u8> => Type::Scalar(Scalar::Bytes),
    #[cfg(feature = "serde_json")]
    serde_json::Value => Type::Scalar(Scalar::Json),
}

macro_rules! spanner_type_defer_to {
    ($($parent:ty: $deferred:ty),* $(,)?) => {
        $(
            impl $crate::ty::SpannerType for $parent {
                const TYPE: &'static Type = <$deferred as $crate::ty::SpannerType>::TYPE;

                const NULLABLE: bool = false;
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
    const TYPE: &'static Type = T::TYPE;

    const NULLABLE: bool = true;
}

macro_rules! impl_for_wrapper_type {
    ($($t:ty),* $(,)?) => {
        $(
            impl<T: SpannerType + ?Sized> SpannerType for $t {
                const TYPE: &'static Type = T::TYPE;
                const NULLABLE: bool = T::NULLABLE;
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
    const TYPE: &'static Type = <T::Owned as SpannerType>::TYPE;
    const NULLABLE: bool = <T::Owned as SpannerType>::NULLABLE;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Struct<S>(pub S);

// TODO flesh out struct conversion
pub trait SpannerStruct: Sized {
    const FIELDS: &'static [Field];

    fn encode(self) -> Result<EncodedStruct<Self>, ConvertError>;

    fn decode(encoded: EncodedStruct<Self>) -> Result<Self, ConvertError>;
}

impl<S: SpannerStruct> SpannerStruct for Struct<S> {
    const FIELDS: &'static [Field] = S::FIELDS;

    fn decode(encoded: EncodedStruct<Self>) -> Result<Self, ConvertError> {
        S::decode(encoded.cast()).map(Self)
    }

    fn encode(self) -> Result<EncodedStruct<Self>, ConvertError> {
        S::encode(self.0).map(EncodedStruct::cast)
    }
}

impl<S: SpannerStruct> crate::FromSpanner for Struct<S> {
    fn from_value(value: crate::Value) -> Result<Self, ConvertError> {
        let values = value.into_array::<Self>()?.values;

        let expected = S::FIELDS.len();
        let count = values.len();

        let fields = EncodedArray::new(values);

        if expected != count {
            let val = crate::Value(Kind::ListValue(ListValue {
                values: fields.values,
            }));

            let err = anyhow::anyhow!("expected {expected} struct fields, recieved {count}");

            return Err(FromError::from_value_and_anyhow::<Self>(val, err).into());
        }

        let encoded = EncodedStruct {
            fields,
            next_field: 0,
            _marker: PhantomData,
        };

        SpannerStruct::decode(encoded).map(Struct)
    }
}

impl<S> crate::convert::SpannerEncode for Struct<S>
where
    S: SpannerStruct,
{
    type Error = ConvertError;
    type SpannerType = EncodedStruct<S>;
    type Encoded = EncodedStruct<S>;

    #[inline]
    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        SpannerStruct::encode(self.0)
    }
}

pub struct EncodedStruct<S: ?Sized> {
    fields: EncodedArray,
    next_field: usize,
    _marker: PhantomData<S>,
}

impl<S> EncodedStruct<S> {
    #[inline]
    fn cast<S2>(self) -> EncodedStruct<S2> {
        EncodedStruct {
            fields: self.fields,
            next_field: self.next_field,
            _marker: PhantomData,
        }
    }
}

impl<S: SpannerStruct + ?Sized> EncodedStruct<S> {
    pub fn pop_column(&mut self) -> Option<crate::Value> {
        let value = self.fields.values.get_mut(self.next_field)?;

        self.next_field += 1;

        Some(crate::Value::from_kind_opt(value.kind.take()))
    }

    pub fn pop_column_with_field(&mut self) -> Option<(&'static Field, crate::Value)> {
        let field = S::FIELDS.get(self.next_field)?;
        self.pop_column().map(|value| (field, value))
    }
}

impl<S: SpannerStruct + ?Sized> SpannerType for EncodedStruct<S> {
    const TYPE: &'static Type = &Type::Struct {
        fields: StaticOrBoxed::Static(S::FIELDS),
    };

    const NULLABLE: bool = false;
}

impl<S: SpannerStruct + ?Sized> crate::IntoSpanner for EncodedStruct<S> {
    fn into_value(self) -> crate::Value {
        crate::Value(Kind::ListValue(ListValue {
            values: self.fields.values,
        }))
    }
}

impl<S: SpannerStruct> SpannerType for Struct<S> {
    const TYPE: &'static Type = &Type::Struct {
        fields: StaticOrBoxed::Static(S::FIELDS),
    };

    const NULLABLE: bool = false;
}

pub struct EncodedProto<T: ?Sized> {
    encoded: bytes::Bytes,
    ty: PhantomData<T>,
}

impl<T: prost::Name + ?Sized> SpannerType for EncodedProto<T> {
    const TYPE: &'static Type = &Type::Proto(ProtoName::Split {
        package: T::PACKAGE,
        name: T::NAME,
    });

    const NULLABLE: bool = false;
}

impl<T: prost::Name + ?Sized> crate::convert::IntoSpanner for EncodedProto<T> {
    #[inline]
    fn into_value(self) -> crate::Value {
        self.encoded.into_value()
    }
}

/// Enum describing the Spanner scalar types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    Bool,
    Int64,
    Float64,
    Timestamp,
    Date,
    String,
    Bytes,
    Numeric,
    Interval,
    Json,
    Enum,
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_spanner_type_str())
    }
}

impl From<Scalar> for Type {
    #[inline]
    fn from(value: Scalar) -> Self {
        Type::Scalar(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Scalar(Scalar),
    Proto(ProtoName),
    Array { element: StaticOrBoxed<Type> },
    Struct { fields: StaticOrBoxed<[Field]> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProtoName {
    FullyQualified(Cow<'static, str>),
    Split {
        package: &'static str,
        name: &'static str,
    },
}

impl ProtoName {
    fn to_fully_qualified(&self) -> String {
        match self {
            Self::FullyQualified(fq) => fq.as_ref().to_owned(),
            Self::Split { package, name } => {
                let mut buf = String::with_capacity(package.len() + 1 + name.len());
                buf.push_str(package);
                buf.push('.');
                buf.push_str(name);
                buf
            }
        }
    }
}

impl fmt::Display for ProtoName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullyQualified(fq) => f.write_str(fq),
            Self::Split { package, name } => {
                f.write_str(package)?;
                f.write_str(".")?;
                f.write_str(name)
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(scalar) => f.write_str(scalar.as_spanner_type_str()),
            Self::Proto(name) => fmt::Display::fmt(name, f),
            Self::Array { element } => {
                f.write_str("ARRAY<")?;
                element.fmt(f)?;
                f.write_str(">")
            }
            Self::Struct { fields } => {
                f.write_str("STRUCT<")?;

                for (idx, field) in fields.iter().enumerate() {
                    f.write_str(&field.name)?;
                    f.write_str(" ")?;
                    field.ty.fmt(f)?;

                    if idx < fields.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                f.write_str(">")
            }
        }
    }
}

impl Type {
    pub const STRING: Self = Self::Scalar(Scalar::String);
    pub const BOOL: Self = Self::Scalar(Scalar::Bool);
    pub const INT64: Self = Self::Scalar(Scalar::Int64);
    pub const FLOAT64: Self = Self::Scalar(Scalar::Float64);
    pub const TIMESTAMP: Self = Self::Scalar(Scalar::Timestamp);
    pub const DATE: Self = Self::Scalar(Scalar::Date);
    pub const BYTES: Self = Self::Scalar(Scalar::Bytes);
    pub const INTERVAL: Self = Self::Scalar(Scalar::Interval);

    pub const NUMERIC: Self = Self::Scalar(Scalar::Numeric);
    pub const JSON: Self = Self::Scalar(Scalar::Json);

    pub(crate) fn to_type_code(&self) -> TypeCode {
        match self {
            Self::Scalar(scalar) => scalar.to_type_code(),
            Self::Proto { .. } => TypeCode::Proto,
            Self::Array { .. } => TypeCode::Array,
            Self::Struct { .. } => TypeCode::Struct,
        }
    }

    pub(crate) fn get_array_elem(&self) -> Option<Cow<'_, Type>> {
        match self {
            Self::Array { element } => Some(Cow::Borrowed(&element)),
            _ => None,
        }
    }

    pub(crate) fn get_struct_fields(&self) -> Option<Cow<'_, [Field]>> {
        match self {
            Self::Struct { fields } => Some(Cow::Borrowed(&fields)),
            _ => None,
        }
    }

    pub(crate) fn from_proto(mut ty: spanner::Type) -> Result<Self, MissingTypeInfo> {
        match TypeCode::try_from(ty.code).unwrap_or(TypeCode::Unspecified) {
            TypeCode::Unspecified => Err(MissingTypeInfo::invalid(ty)),
            TypeCode::Bool => Ok(Self::Scalar(Scalar::Bool)),
            TypeCode::Int64 => Ok(Self::Scalar(Scalar::Int64)),
            TypeCode::Float64 | TypeCode::Float32 => Ok(Self::Scalar(Scalar::Float64)),
            TypeCode::Json => Ok(Self::Scalar(Scalar::Json)),
            TypeCode::Date => Ok(Self::Scalar(Scalar::Date)),
            TypeCode::Timestamp => Ok(Self::Scalar(Scalar::Timestamp)),
            TypeCode::String => Ok(Self::Scalar(Scalar::String)),
            TypeCode::Bytes => Ok(Self::Scalar(Scalar::Bytes)),
            TypeCode::Numeric => Ok(Self::Scalar(Scalar::Numeric)),
            TypeCode::Proto => Ok(Self::Proto(ProtoName::FullyQualified(Cow::Owned(
                ty.proto_type_fqn,
            )))),
            TypeCode::Array => {
                let raw_elem = ty
                    .array_element_type
                    .take()
                    .ok_or_else(|| MissingTypeInfo::invalid(ty))?;

                let element = Self::from_proto(*raw_elem)?;

                Ok(Self::Array {
                    element: StaticOrBoxed::Boxed(Box::new(element)),
                })
            }
            TypeCode::Interval => Ok(Self::Scalar(Scalar::Interval)),
            TypeCode::Struct => {
                let raw_struct = ty
                    .struct_type
                    .take()
                    .ok_or_else(|| MissingTypeInfo::invalid(ty))?;

                let fields = raw_struct
                    .fields
                    .into_iter()
                    .map(Field::from_proto)
                    .collect::<Result<Vec<Field>, MissingTypeInfo>>()?;

                Ok(Self::Struct {
                    fields: StaticOrBoxed::Boxed(fields.into_boxed_slice()),
                })
            }
            // TODO: figure out these new types
            TypeCode::Enum => Ok(Self::Scalar(Scalar::Enum)),
        }
    }

    pub(crate) fn into_proto(&self) -> spanner::Type {
        match self {
            Self::Scalar(scalar) => scalar.into_proto(),
            Self::Proto(proto_name) => spanner::Type {
                code: TypeCode::Proto as i32,
                proto_type_fqn: proto_name.to_fully_qualified(),
                array_element_type: None,
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified as i32,
            },
            Self::Array { element } => spanner::Type {
                code: TypeCode::Array as i32,
                proto_type_fqn: String::new(),
                array_element_type: Some(Box::new(element.into_proto())),
                struct_type: None,
                type_annotation: TypeAnnotationCode::Unspecified as i32,
            },
            Self::Struct { fields } => spanner::Type {
                code: TypeCode::Struct as i32,
                proto_type_fqn: String::new(),
                array_element_type: None,
                struct_type: Some(StructType {
                    fields: Field::into_struct_fields(fields),
                }),
                type_annotation: TypeAnnotationCode::Unspecified as i32,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field {
    pub ty: Type,
    pub name: Cow<'static, str>,
}

impl Field {
    pub(crate) fn from_proto(field: spanner::struct_type::Field) -> Result<Self, MissingTypeInfo> {
        let ty = Type::from_proto(field.r#type.unwrap_or_default())?;

        Ok(Self {
            ty,
            name: Cow::Owned(field.name),
        })
    }

    fn into_struct_fields(fields: &[Self]) -> Vec<struct_type::Field> {
        let mut dst = Vec::with_capacity(fields.len());
        dst.extend(fields.iter().map(|f| f.clone().into_proto()));
        dst
    }

    pub(crate) fn into_proto(self) -> struct_type::Field {
        struct_type::Field {
            name: self.name.into_owned(),
            r#type: Some(self.ty.into_proto()),
        }
    }
}

impl Scalar {
    #[inline]
    pub(crate) const fn into_proto(self) -> spanner::Type {
        spanner::Type {
            proto_type_fqn: String::new(),
            code: self.to_type_code() as i32,
            array_element_type: None,
            struct_type: None,
            type_annotation: TypeAnnotationCode::Unspecified as i32,
        }
    }

    #[inline]
    pub(crate) const fn to_type_code(self) -> TypeCode {
        match self {
            Self::Interval => TypeCode::Interval,
            Self::Bool => TypeCode::Bool,
            Self::Int64 => TypeCode::Int64,
            Self::Float64 => TypeCode::Float64,
            Self::Timestamp => TypeCode::Timestamp,
            Self::Date => TypeCode::Date,
            Self::String => TypeCode::String,
            Self::Bytes => TypeCode::Bytes,
            Self::Numeric => TypeCode::Numeric,
            Self::Json => TypeCode::Json,
            Self::Enum => TypeCode::Enum,
        }
    }

    const fn as_spanner_type_str(self) -> &'static str {
        match self {
            Self::Interval => "INTERVAL",
            Self::Bool => "BOOL",
            Self::Int64 => "INT64",
            Self::Float64 => "FLOAT64",
            Self::Date => "DATE",
            Self::Timestamp => "TIMESTAMP",
            Self::String => "STRING",
            Self::Bytes => "BYTES",
            Self::Numeric => "NUMERIC",
            Self::Json => "JSON",
            Self::Enum => "ENUM",
        }
    }
}
