use std::fmt;
use std::marker::PhantomData;

use protos::protobuf::{self, ListValue};

use crate::convert::SpannerEncode;
use crate::ty::SpannerType;
use crate::{IntoSpanner, Value};

#[derive(Clone, PartialEq)]
#[repr(transparent)]
pub struct EncodedArray<T = Value> {
    pub(crate) values: Vec<protobuf::Value>,
    _marker: PhantomData<T>,
}

impl<T> SpannerType for EncodedArray<T>
where
    T: SpannerType,
{
    type Nullable = typenum::False;
    type Type = crate::ty::markers::Array<T::Type>;
}

impl<T: SpannerType> IntoSpanner for EncodedArray<T> {
    #[inline]
    fn into_value(self) -> crate::Value {
        Value(protobuf::value::Kind::ListValue(protobuf::ListValue {
            values: self.values,
        }))
    }
}

impl<T: SpannerType> crate::FromSpanner for EncodedArray<T> {
    fn from_value(value: Value) -> Result<Self, crate::error::ConvertError> {
        Ok(Self {
            values: value.into_array::<Self>()?.values,
            _marker: PhantomData,
        })
    }
}

fn encode_proto_value<T>(value: T) -> Result<protobuf::Value, T::Error>
where
    T: SpannerEncode,
{
    value.encode().map(to_proto_value)
}

impl<T: SpannerType> EncodedArray<T> {
    pub fn encode_from<I>(src: I) -> Result<Self, <I::Item as SpannerEncode>::Error>
    where
        I: IntoIterator,
        I::Item: SpannerEncode,
    {
        let values = src
            .into_iter()
            .map(encode_proto_value)
            .collect::<Result<Vec<protos::protobuf::Value>, _>>()?;

        Ok(Self {
            values,
            _marker: PhantomData,
        })
    }
}

impl<T> EncodedArray<T> {
    #[inline]
    pub(crate) fn new(values: Vec<protobuf::Value>) -> Self {
        Self {
            values,
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for EncodedArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use super::fmt_helpers::DebugValue;

        let mut dbg = f.debug_list();

        for val in self.values.iter() {
            match val.kind.as_ref() {
                Some(kind) => dbg.entry(&DebugValue(kind)),
                None => dbg.entry(&DebugValue(&super::Value::NULL.0)),
            };
        }

        dbg.finish()
    }
}

impl From<ListValue> for EncodedArray {
    #[inline]
    fn from(lv: ListValue) -> Self {
        Self::from(lv.values)
    }
}

impl From<Vec<protobuf::Value>> for EncodedArray {
    #[inline]
    fn from(values: Vec<protobuf::Value>) -> Self {
        Self {
            values,
            _marker: PhantomData,
        }
    }
}

impl<T> From<Vec<T>> for EncodedArray<T>
where
    T: IntoSpanner,
{
    #[inline]
    fn from(values: Vec<T>) -> Self {
        values.into_iter().collect()
    }
}

#[inline]
fn to_proto_value<T: IntoSpanner>(value: T) -> protobuf::Value {
    value.into_value().into_protobuf()
}

impl<T: IntoSpanner> FromIterator<T> for EncodedArray<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            values: iter.into_iter().map(to_proto_value).collect(),
            _marker: PhantomData,
        }
    }
}

impl<T: IntoSpanner> Extend<T> for EncodedArray<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.values.extend(iter.into_iter().map(to_proto_value));
    }
}
