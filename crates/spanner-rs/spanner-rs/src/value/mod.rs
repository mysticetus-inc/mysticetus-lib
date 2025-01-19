use std::fmt;
use std::marker::PhantomData;

use protos::protobuf::value::Kind;
use protos::protobuf::{self, NullValue};
use protos::spanner::TypeCode;

mod encoded_array;
mod encoded_value;
mod struct_value;

pub use encoded_array::EncodedArray;
pub use encoded_value::EncodedValue;

use crate::Table;
use crate::convert::SpannerEncode;
use crate::error::{ConvertError, FromError};

/// A Spanner value.
#[derive(Clone, PartialEq)]
#[repr(transparent)]
pub struct Value(pub(crate) Kind);

impl fmt::Debug for Value {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_helpers::DebugValue(&self.0).fmt(f)
    }
}

impl From<Kind> for Value {
    fn from(kind: Kind) -> Self {
        Self(kind)
    }
}

impl From<protobuf::Value> for Value {
    fn from(value: protobuf::Value) -> Self {
        match value.kind {
            None => Value::NULL,
            Some(kind) => Value(kind),
        }
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::NULL,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self(Kind::StringValue(value))
    }
}

impl From<Vec<Value>> for Value {
    fn from(values: Vec<Value>) -> Self {
        // TODO: see it the fast path of a transmute is safe
        // let values = unsafe { std::mem::transmute(values) };
        let values = values.into_iter().map(Value::into_protobuf).collect();

        Self(Kind::ListValue(protobuf::ListValue { values }))
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self(Kind::BoolValue(value))
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(value: f64) -> Self {
        use std::num::FpCategory::*;

        match value.classify() {
            Zero | Subnormal | Normal => Value(Kind::NumberValue(value)),
            Nan => String::from("NaN").into(),
            Infinite if value.is_sign_positive() => String::from("Infinity").into(),
            Infinite => String::from("-Infinity").into(),
        }
    }
}

macro_rules! impl_is_type_fns {
    ($($fn_name:ident($variant:tt)),* $(,)?) => {
        $(
            pub const fn $fn_name(&self) -> bool {
                matches!(self.0, Kind::$variant(_))
            }
        )*
    };
}

macro_rules! impl_into_type_fns {
    ($($visib:vis $fn_name:ident($variant:tt) -> $out:ty),* $(,)?) => {
        $(
            $visib fn $fn_name<Expecting: crate::convert::FromSpanner>(self) -> Result<$out, FromError> {
                match self.0 {
                    Kind::$variant(val) => Ok(val),
                    other => Err(FromError::from_value::<Expecting>(Value(other))),
                }
            }
        )*
    };
}

impl Value {
    impl_is_type_fns! {
        is_null(NullValue),
        is_array(ListValue),
        is_bool(BoolValue),
        is_string(StringValue),
        is_number(NumberValue),
        is_struct(StructValue),
    }

    impl_into_type_fns! {
        // dont want to expose protobuf types, so these are crate only
        pub(crate) into_array(ListValue) -> protos::protobuf::ListValue,
        pub(crate) into_struct(StructValue) -> protos::protobuf::Struct,
        pub into_bool(BoolValue) -> bool,
        pub into_string(StringValue) -> String,
        pub into_number(NumberValue) -> f64,
    }

    #[cfg(feature = "serde_json")]
    pub fn json<T: serde::Serialize>(value: &T) -> Result<Self, crate::error::IntoError> {
        match serde_json::to_string(value) {
            Ok(s) => Ok(Self::from(s)),
            Err(err) => {
                Err(crate::error::IntoError::from_error(err).reason("failed to serialize JSON"))
            }
        }
    }
}

/// An opaque row of values, from the Spanner API. We need to use this instead
/// of Vec<Value>, because the protobuf Value is different enough that we'd need
/// to re-allocate a new Vec.
#[repr(transparent)]
pub struct Row(pub(crate) Vec<protobuf::Value>);

impl From<Vec<protobuf::Value>> for Row {
    fn from(value: Vec<protobuf::Value>) -> Self {
        Self(value)
    }
}

impl Value {
    pub const NULL: Self = Self(Kind::NullValue(NullValue::NullValue as i32));

    pub(crate) fn from_proto(v: protobuf::Value) -> Self {
        v.kind.map(Self).unwrap_or(Value::NULL)
    }

    #[inline]
    pub(crate) const fn from_kind(kind: Kind) -> Self {
        Self(kind)
    }

    #[inline]
    pub(crate) fn from_kind_opt(kind: Option<Kind>) -> Self {
        kind.map(Self).unwrap_or(Self::NULL)
    }

    pub(crate) fn fallback_type_code(&self) -> TypeCode {
        match &self.0 {
            Kind::NullValue(_) => TypeCode::Unspecified,
            Kind::NumberValue(_) => TypeCode::Float64,
            Kind::StringValue(_) => TypeCode::String,
            Kind::BoolValue(_) => TypeCode::Bool,
            Kind::StructValue(_) => TypeCode::Struct,
            Kind::ListValue(_) => TypeCode::Array,
        }
    }

    #[doc(hidden)]
    pub fn from_protobuf(proto: protobuf::Value) -> Self {
        match proto.kind {
            Some(kind) => Self(kind),
            _ => Self::NULL,
        }
    }

    #[doc(hidden)]
    pub fn into_protobuf(self) -> protobuf::Value {
        protobuf::Value { kind: Some(self.0) }
    }
}

pub struct RowBuilder<T> {
    _marker: PhantomData<T>,
    filled: usize,
    dst: Vec<protobuf::Value>,
}

impl<T: Table> RowBuilder<T> {
    pub fn add_column<E: SpannerEncode>(&mut self, value: E) -> Result<(), ConvertError> {
        let value = value.encode_to_value().map_err(Into::into)?;
        self.add_column_value(value);
        Ok(())
    }

    pub fn serialize_column<V: serde::Serialize>(&mut self, value: V) -> Result<(), ConvertError> {
        let value = crate::serde::ValueSerializer::serialize(value)?;
        self.add_column_value(value);
        Ok(())
    }

    #[inline]
    pub fn add_column_value(&mut self, column: Value) {
        assert_ne!(self.filled, T::COLUMNS.len(), "row already fully populated");
        self.dst.push(column.into_protobuf());
        self.filled += 1;
    }

    #[inline]
    pub fn build(self) -> Row {
        assert_eq!(self.filled, T::COLUMNS.len(), "row already fully populated");
        Row(self.dst)
    }
}

impl<T: Table> Default for RowBuilder<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
            dst: Vec::with_capacity(T::COLUMNS.len()),
            filled: 0,
        }
    }
}

impl Row {
    #[inline]
    pub fn builder<T: Table>() -> RowBuilder<T> {
        RowBuilder::default()
    }

    pub fn cols(&self) -> usize {
        self.0.len()
    }

    pub fn take(&mut self, index: usize) -> Value {
        self.0
            .get_mut(index)
            .and_then(|value| value.kind.take())
            .map(Value)
            .unwrap_or(Value::NULL)
    }
}

/// Formatting helpers to unwrap some of the many, many layers of protobuf values
/// to avoid bloating logs.
pub(crate) mod fmt_helpers {
    use std::collections::HashMap;
    use std::fmt;

    use protos::protobuf;
    use protos::protobuf::value::Kind;

    pub(crate) struct DebugMap<'a>(pub &'a HashMap<String, protobuf::Value>);

    pub(crate) struct DebugList<'a>(pub &'a [protobuf::Value]);

    pub(crate) struct DebugValue<'a>(pub &'a Kind);

    impl fmt::Debug for DebugMap<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let entries = self
                .0
                .iter()
                .filter_map(|(key, value)| value.kind.as_ref().map(|k| (key, DebugValue(k))));

            f.debug_map().entries(entries).finish()
        }
    }

    impl fmt::Debug for DebugList<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let entries = self
                .0
                .iter()
                .filter_map(|value| value.kind.as_ref().map(DebugValue));

            f.debug_list().entries(entries).finish()
        }
    }

    impl fmt::Debug for DebugValue<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            macro_rules! fmt {
                ($f:expr; $name:literal $(, $val:expr)?) => {
                    $f.debug_tuple($name)
                    $(.field(&$val))?
                        .finish()
                };
            }

            match &self.0 {
                Kind::NullValue(_) => fmt!(f; "Null"),
                Kind::BoolValue(b) => fmt!(f; "Bool", b),
                Kind::NumberValue(n) => fmt!(f; "Number", n),
                Kind::StringValue(s) => fmt!(f; "String", s),
                Kind::StructValue(map) => fmt!(f; "Map", DebugMap(&map.fields)),
                Kind::ListValue(list) => fmt!(f; "List", DebugList(&list.values)),
            }
        }
    }
}

#[test]
fn test_value_repr_compatible_with_proto_value() {
    assert_eq!(
        std::mem::size_of::<Value>(),
        std::mem::size_of::<protobuf::Value>(),
    );

    assert_eq!(
        std::mem::align_of::<Value>(),
        std::mem::align_of::<protobuf::Value>(),
    );
    let start = Value::from("test".to_owned());
    let clone = start.clone();
    let end = unsafe { std::mem::transmute::<Value, protobuf::Value>(start) };

    assert_eq!(end.kind.unwrap(), clone.0);
}
