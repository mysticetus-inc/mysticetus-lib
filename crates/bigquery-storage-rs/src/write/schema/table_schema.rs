use bigquery_resources_rs::table::{self as rest_table, FieldMode, FieldType};
use protos::bigquery_storage as proto_table;
use protos::bigquery_storage::table_field_schema::{Mode, Type};

pub trait TableSchema: Sized {
    type Fields: IntoIterator<IntoIter: ExactSizeIterator, Item = Self::Field>;
    type Field: FieldSchema;

    fn into_fields(self) -> Self::Fields;
}

impl TableSchema for proto_table::TableSchema {
    type Field = proto_table::TableFieldSchema;
    type Fields = Vec<proto_table::TableFieldSchema>;

    #[inline]
    fn into_fields(self) -> Self::Fields {
        self.fields
    }
}

impl<const N: usize, S> TableSchema for [rest_table::TableFieldSchema<S>; N]
where
    S: Into<Box<str>>,
{
    type Field = rest_table::TableFieldSchema<S>;
    type Fields = Self;

    #[inline]
    fn into_fields(self) -> Self::Fields {
        self
    }
}

impl<S> TableSchema for Vec<rest_table::TableFieldSchema<S>>
where
    S: Into<Box<str>>,
{
    type Field = rest_table::TableFieldSchema<S>;
    type Fields = Self;

    #[inline]
    fn into_fields(self) -> Self::Fields {
        self
    }
}

impl<S> TableSchema for Box<[rest_table::TableFieldSchema<S>]>
where
    S: Into<Box<str>>,
{
    type Field = rest_table::TableFieldSchema<S>;
    type Fields = Vec<Self::Field>;

    #[inline]
    fn into_fields(self) -> Self::Fields {
        self.into_vec()
    }
}

pub trait FieldSchema {
    type Name: Into<Box<str>>;

    fn ty(&self) -> Option<FieldType>;

    fn proto_ty(&self) -> Type;

    fn mode(&self) -> Option<FieldMode>;

    fn proto_mode(&self) -> Mode;

    fn into_field_name(self) -> Self::Name;
}

impl<S: Into<Box<str>>> FieldSchema for rest_table::TableFieldSchema<S> {
    type Name = S;

    #[inline]
    fn ty(&self) -> Option<FieldType> {
        Some(self.ty)
    }

    #[inline]
    fn proto_ty(&self) -> Type {
        field_type_to_proto_type(self.ty)
    }

    #[inline]
    fn mode(&self) -> Option<FieldMode> {
        Some(self.mode)
    }

    #[inline]
    fn proto_mode(&self) -> Mode {
        field_mode_to_proto_mode(self.mode)
    }

    #[inline]
    fn into_field_name(self) -> Self::Name {
        self.name
    }
}

impl FieldSchema for proto_table::TableFieldSchema {
    type Name = String;

    #[inline]
    fn ty(&self) -> Option<FieldType> {
        proto_type_to_field_type(self.r#type())
    }

    #[inline]
    fn proto_ty(&self) -> Type {
        self.r#type()
    }

    #[inline]
    fn mode(&self) -> Option<FieldMode> {
        proto_mode_to_field_mode(self.mode())
    }

    #[inline]
    fn proto_mode(&self) -> Mode {
        self.mode()
    }

    #[inline]
    fn into_field_name(self) -> Self::Name {
        self.name
    }
}

pub const fn proto_mode_to_field_mode(mode: Mode) -> Option<FieldMode> {
    match mode {
        Mode::Nullable => Some(FieldMode::Nullable),
        Mode::Repeated => Some(FieldMode::Repeated),
        Mode::Required => Some(FieldMode::Required),
        Mode::Unspecified => None,
    }
}

pub const fn field_mode_to_proto_mode(mode: FieldMode) -> Mode {
    match mode {
        FieldMode::Nullable => Mode::Nullable,
        FieldMode::Repeated => Mode::Repeated,
        FieldMode::Required => Mode::Required,
    }
}

pub const fn proto_type_to_field_type(ty: Type) -> Option<FieldType> {
    match ty {
        Type::Unspecified => None,
        Type::String => Some(FieldType::String),
        Type::Int64 => Some(FieldType::Integer),
        Type::Double => Some(FieldType::Float),
        Type::Struct => Some(FieldType::Record),
        Type::Bytes => Some(FieldType::Bytes),
        Type::Bool => Some(FieldType::Bool),
        Type::Timestamp => Some(FieldType::Timestamp),
        Type::Date => Some(FieldType::Date),
        Type::Time => Some(FieldType::Time),
        Type::Datetime => Some(FieldType::DateTime),
        Type::Geography => Some(FieldType::Geography),
        Type::Numeric => Some(FieldType::Numeric),
        Type::Bignumeric => Some(FieldType::BigNumeric),
        Type::Interval => Some(FieldType::Interval),
        Type::Json => Some(FieldType::Json),
        Type::Range => Some(FieldType::Range),
    }
}

pub const fn field_type_to_proto_type(ty: FieldType) -> Type {
    match ty {
        FieldType::String => Type::String,
        FieldType::Bytes => Type::Bytes,
        FieldType::Integer => Type::Int64,
        FieldType::Float => Type::Double,
        FieldType::Bool => Type::Bool,
        FieldType::Timestamp => Type::Timestamp,
        FieldType::Date => Type::Date,
        FieldType::Time => Type::Time,
        FieldType::DateTime => Type::Datetime,
        FieldType::Geography => Type::Geography,
        FieldType::Numeric => Type::Numeric,
        FieldType::BigNumeric => Type::Bignumeric,
        FieldType::Json => Type::Json,
        FieldType::Record => Type::Struct,
        FieldType::Range => Type::Range,
        FieldType::Interval => Type::Interval,
    }
}
