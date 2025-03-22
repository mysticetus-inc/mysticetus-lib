use std::marker::PhantomData;

use base64::Engine;

use super::Value;
use crate::error::{ConvertError, FromError};
use crate::ty::SpannerType;
use crate::ty::markers::{Proto, SpannerProto};
use crate::{FromSpanner, IntoSpanner};

pub struct EncodedProto<T: ?Sized> {
    encoded: bytes::Bytes,
    ty: PhantomData<T>,
}

impl<T: prost::Name> EncodedProto<T> {
    pub fn decode(&self) -> Result<T, prost::DecodeError>
    where
        T: Default,
    {
        <T as prost::Message>::decode(&mut &self.encoded[..])
    }

    pub fn encode(value: &T) -> Self {
        Self {
            encoded: value.encode_to_vec().into(),
            ty: PhantomData,
        }
    }
}

impl<T: prost::Name + ?Sized + SpannerProto> SpannerType for EncodedProto<T> {
    type Nullable = typenum::False;
    type Type = Proto<T>;
}

impl<T: prost::Name + ?Sized + SpannerProto> IntoSpanner for EncodedProto<T> {
    #[inline]
    fn into_value(self) -> Value {
        self.encoded.into_value()
    }
}

impl<T: prost::Name + ?Sized + SpannerProto> FromSpanner for EncodedProto<T> {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let encoded_string = value
            .into_string::<Self>()
            .map_err(|from_err| from_err.with_type::<Self>())?;

        let proto_encoded = base64::engine::general_purpose::STANDARD
            .decode(encoded_string.as_bytes())
            .map_err(|err| FromError::from_value_and_error::<Self>(encoded_string, err))?;

        Ok(Self {
            encoded: bytes::Bytes::from(proto_encoded),
            ty: PhantomData,
        })
    }
}
