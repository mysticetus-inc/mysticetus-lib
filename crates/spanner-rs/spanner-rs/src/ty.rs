use std::borrow::Cow;
use std::fmt;

use protos::spanner::{self, StructType, TypeAnnotationCode, TypeCode, struct_type};
// re-export for downstream consumers
pub use shared::static_or_boxed::StaticOrBoxed;
pub use typenum::{False, True};

pub mod markers;
mod spanner_type;
pub use spanner_type::SpannerType;
pub(crate) use spanner_type::{nullable, ty};

use crate::error::MissingTypeInfo;

/// Information about a Spanner type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Scalar(Scalar),
    Proto(ProtoName),
    Enum(ProtoName),
    Array { element: StaticOrBoxed<Type> },
    Struct { fields: StaticOrBoxed<[Field]> },
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
            Self::Proto(name) => write!(f, "PROTO({name})"),
            Self::Enum(name) => write!(f, "ENUM({name})"),
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

    #[inline]
    pub const fn array(element: &'static Self) -> Self {
        Self::Array {
            element: StaticOrBoxed::Static(element),
        }
    }

    #[inline]
    pub const fn struct_type(fields: &'static [Field]) -> Self {
        Self::Struct {
            fields: StaticOrBoxed::Static(fields),
        }
    }

    #[inline]
    pub const fn proto<T: markers::SpannerProto + ?Sized>() -> Self {
        Self::Proto(ProtoName::Split {
            package: T::PACKAGE,
            name: T::NAME,
        })
    }

    #[inline]
    pub const fn proto_enum<T: markers::SpannerEnum + ?Sized>() -> Self {
        Self::Enum(ProtoName::Split {
            package: T::PACKAGE,
            name: T::NAME,
        })
    }

    pub(crate) fn to_type_code(&self) -> TypeCode {
        match self {
            Self::Scalar(scalar) => scalar.to_type_code(),
            Self::Proto(_) => TypeCode::Proto,
            Self::Enum(_) => TypeCode::Enum,
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
            TypeCode::Enum => Ok(Self::Enum(ProtoName::FullyQualified(Cow::Owned(
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
            Self::Enum(enum_name) => spanner::Type {
                code: TypeCode::Enum as i32,
                proto_type_fqn: enum_name.to_fully_qualified(),
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
    #[inline]
    pub const fn new(ty: Type, name: &'static str) -> Self {
        Self {
            ty,
            name: Cow::Borrowed(name),
        }
    }

    pub(crate) fn from_proto(field: spanner::struct_type::Field) -> Result<Self, MissingTypeInfo> {
        let ty = Type::from_proto(field.r#type.unwrap_or_default())?;

        Ok(Self {
            ty,
            name: Cow::Owned(field.name),
        })
    }

    fn into_struct_fields(fields: &[Self]) -> Vec<struct_type::Field> {
        let mut dst = Vec::with_capacity(fields.len());
        dst.extend(fields.iter().map(Self::into_proto));
        dst
    }

    pub(crate) fn into_proto(&self) -> struct_type::Field {
        struct_type::Field {
            name: match self.name {
                Cow::Owned(ref owned) => owned.clone(),
                Cow::Borrowed(s) => s.to_owned(),
            },
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
        }
    }
}
