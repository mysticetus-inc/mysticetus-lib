use std::marker::PhantomData;

use crate::convert::SpannerEncode;
use crate::error::ConvertError;
use crate::ty::SpannerType;
use crate::{FromSpanner, IntoSpanner, Value};

/// A transparent wrapper around [`Value`] that retains type information, allowing it to implement
/// [`IntoSpanner`]. Useful in situations where implementing [`SpannerEncode`] runs into conflicts.
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct EncodedValue<T> {
    value: super::Value,
    _marker: PhantomData<fn(T)>,
}

impl<T> EncodedValue<Option<T>> {
    pub fn encode_option(value: impl IntoSpanner<SpannerType = T>) -> Self {
        Self::new(value.into_value())
    }
}

impl<T> EncodedValue<T> {
    pub const NULL: Self = Self {
        value: Value::NULL,
        _marker: PhantomData,
    };

    #[inline]
    pub const fn new(value: Value) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub fn encode(value: impl IntoSpanner<SpannerType = T>) -> Self {
        Self::new(value.into_value())
    }

    #[inline]
    pub const fn get(&self) -> &Value {
        &self.value
    }

    #[inline]
    pub fn into_value(self) -> Value {
        self.value
    }

    #[inline]
    pub const fn get_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn decode<Decoded>(self) -> Result<Decoded, ConvertError>
    where
        Decoded: FromSpanner<SpannerType = T>,
    {
        FromSpanner::from_value(self.value)
    }
}

impl<T> From<Value> for EncodedValue<T> {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl<T: SpannerType> SpannerType for EncodedValue<T> {
    type Type = T::Type;
    type Nullable = T::Nullable;
}

impl<T: SpannerType> IntoSpanner for EncodedValue<T> {
    type SpannerType = T;
    #[inline]
    fn into_value(self) -> crate::Value {
        self.value
    }
}

impl<T: SpannerType> FromSpanner for EncodedValue<T> {
    type SpannerType = Self;

    fn from_value(value: Value) -> Result<Self, ConvertError> {
        Ok(Self::new(value))
    }
}
