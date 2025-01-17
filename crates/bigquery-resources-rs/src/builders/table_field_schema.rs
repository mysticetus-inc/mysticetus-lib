use std::num::{NonZeroI64, NonZeroUsize};

use super::Unset;
use crate::table::{FieldMode, FieldType, RangeElementType, RoundingMode, TableFieldSchema};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableFieldSchemaBuilder<S, Ty> {
    name: S,
    ty: Ty,
    // optional fields
    rounding_mode: Option<RoundingMode>,
    range_element_type: Option<RangeElementType>,
    max_length: Option<NonZeroUsize>,
    description: Option<S>,
    default_value_expression: Option<S>,
    scale: Option<NonZeroI64>,
    precision: Option<NonZeroI64>,
}

impl<S> TableFieldSchemaBuilder<S, Unset> {
    pub(crate) const fn new(name: S) -> Self {
        Self {
            name,
            ty: Unset,
            rounding_mode: None,
            range_element_type: None,
            max_length: None,
            description: None,
            default_value_expression: None,
            scale: None,
            precision: None,
        }
    }
}

macro_rules! define_ty_builder_fn {
    ($($name:ident($ty_variant:ident)),* $(,)?) => {
        $(
            #[inline]
            pub fn $name(self) -> TableFieldSchemaBuilder<S, FieldType> {
                self.with_type(FieldType::$ty_variant)
            }
        )*
    };
}

impl<S> TableFieldSchemaBuilder<S, Unset> {
    fn with_type(self, ty: FieldType) -> TableFieldSchemaBuilder<S, FieldType> {
        TableFieldSchemaBuilder {
            name: self.name,
            ty,
            rounding_mode: self.rounding_mode,
            range_element_type: self.range_element_type,
            max_length: self.max_length,
            description: self.description,
            default_value_expression: self.default_value_expression,
            scale: self.scale,
            precision: self.precision,
        }
    }

    define_ty_builder_fn! {
        string(String),
        bytes(Bytes),
        int(Integer),
        float(Float),
        geography(Geography),
        json(Json),
        record(Record),
        bool(Bool),
        timestamp(Timestamp),
        time(Time),
        date(Date),
        datetime(DateTime),
        // remaining types are intentionally ignored for the moment
    }
}

macro_rules! define_mode_builder_fn {
    ($($name:ident($mode_variant:ident)),* $(,)?) => {
        $(
            #[inline]
            pub fn $name(self) -> TableFieldSchema<S> {
                self.build_with_mode(FieldMode::$mode_variant)
            }
        )*
    };
}

impl<S> TableFieldSchemaBuilder<S, FieldType> {
    fn build_with_mode(self, mode: FieldMode) -> TableFieldSchema<S> {
        TableFieldSchema {
            name: self.name,
            ty: self.ty,
            mode,
            rounding_mode: self.rounding_mode,
            range_element_type: self.range_element_type,
            max_length: self.max_length,
            description: self.description,
            default_value_expression: self.default_value_expression,
            scale: self.scale,
            precision: self.precision,
        }
    }

    define_mode_builder_fn! {
        required(Required),
        repeated(Repeated),
        nullable(Nullable),
    }
}
