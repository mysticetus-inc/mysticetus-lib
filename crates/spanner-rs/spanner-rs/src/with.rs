//! <code>#[spanner(with = ...)]</code> adapter types.

use bytes::Bytes;
use protos::protobuf::value::Kind;

use crate::error::{ConvertError, FromError};
use crate::ty::SpannerType;
use crate::{FromSpanner, IntoSpanner, Value};

// -------------------- Json<T> ------------------ //

#[cfg(feature = "serde_json")]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Json<T>(pub T);

#[cfg(feature = "serde_json")]
pub struct JsonString<T>(String, std::marker::PhantomData<T>);

#[cfg(feature = "serde_json")]
impl<T> IntoSpanner for JsonString<T> {
    type SpannerType = Json<T>;

    #[inline]
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

#[cfg(feature = "serde_json")]
impl<T> SpannerType for Json<T> {
    type Type = crate::ty::markers::Json;
    type Nullable = typenum::True;
}

#[cfg(feature = "serde_json")]
impl<T> crate::convert::SpannerEncode for Json<T>
where
    T: serde::Serialize,
{
    type SpannerType = Self;

    type Encoded = JsonString<T>;

    type Error = crate::error::IntoError;

    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        match serde_json::to_string(&self.0) {
            Ok(encoded) => Ok(JsonString(encoded, std::marker::PhantomData)),
            Err(err) => Err(crate::error::IntoError::from_error(err)),
        }
    }
}

#[cfg(feature = "serde_json")]
impl<T> crate::convert::SpannerEncode for &Json<T>
where
    T: serde::Serialize,
{
    type SpannerType = Json<T>;

    type Encoded = JsonString<T>;

    type Error = crate::error::IntoError;

    #[inline]
    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        Json(&self.0)
            .encode()
            .map(|JsonString(s, _)| JsonString(s, std::marker::PhantomData))
    }
}

#[cfg(feature = "serde_json")]
impl<T: serde::Serialize + serde::de::DeserializeOwned> FromSpanner for Json<T> {
    type SpannerType = Self;

    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = match value.0 {
            Kind::StringValue(s) => s,
            other => {
                return Err(FromError::from_value::<Self>(Value(other)).into());
            }
        };

        match serde_json::from_str(&s) {
            Ok(value) => Ok(Json(value)),
            Err(err) => Err(FromError::from_value_and_error::<Self>(s, err).into()),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Array<I>(pub I);

impl<I> SpannerType for Array<I>
where
    I: IntoIterator,
    I::Item: SpannerType,
{
    type Type = crate::ty::markers::Array<<I::Item as SpannerType>::Type>;
    type Nullable = typenum::False;
}

// -------------- Bytes ------------------ //

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AsBytes<T = Bytes>(pub T);

impl<T: AsRef<[u8]>> SpannerType for AsBytes<T> {
    type Type = crate::ty::markers::Bytes;
    type Nullable = typenum::False;
}

// inner encode/decode function with no generic params to avoid
// tons of generated/monomorphized versions that covers tons of
// concrete types (because the actual `base64` method takes
// an `impl AsRef<[u8]>` instead of `&[u8]`)

#[inline]
fn encode(bytes: &[u8]) -> String {
    use base64::Engine;

    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[inline]
fn decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;

    base64::engine::general_purpose::STANDARD.decode(s.as_bytes())
}

impl<T: AsRef<[u8]>> IntoSpanner for AsBytes<T> {
    type SpannerType = Self;

    fn into_value(self) -> Value {
        encode(self.0.as_ref()).into_value()
    }
}

impl<T: AsRef<[u8]> + From<Vec<u8>>> FromSpanner for AsBytes<T> {
    type SpannerType = Bytes;

    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = value.into_string::<Self>()?;

        match decode(&s) {
            Ok(vec) => Ok(AsBytes(T::from(vec))),
            Err(err) => Err(FromError::from_value_and_error::<Self>(s, err).into()),
        }
    }
}

// =================== Proto =======================

pub struct Proto<T>(pub T);

impl<T: prost::Name + crate::ty::markers::SpannerProto> SpannerType for Proto<T> {
    type Type = crate::ty::markers::Proto<T>;
    type Nullable = typenum::False;
}

impl<T: prost::Name + crate::ty::markers::SpannerProto> IntoSpanner for Proto<T> {
    type SpannerType = Self;

    fn into_value(self) -> Value {
        encode(&self.0.encode_to_vec()).into_value()
    }
}

impl<T: prost::Name + crate::ty::markers::SpannerProto + Default> FromSpanner for Proto<T> {
    type SpannerType = Self;
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let string = value.into_string::<Self>()?;

        let proto_bytes = match decode(&string) {
            Ok(vec) => vec,
            Err(err) => return Err(FromError::from_value_and_error::<Self>(string, err).into()),
        };

        match T::decode(&mut &proto_bytes[..]) {
            Ok(decoded) => Ok(Self(decoded)),
            Err(err) => Err(FromError::from_value_and_error::<Self>(string, err).into()),
        }
    }
}

// ================ Proto buf Enum --===============

pub struct AsProtoEnum<T>(pub T);

impl<T: Into<i32> + TryFrom<i32> + crate::ty::markers::SpannerProto> SpannerType
    for AsProtoEnum<T>
{
    type Type = crate::ty::markers::Proto<T>;
    type Nullable = typenum::False;
}

impl<T: Into<i32> + TryFrom<i32> + crate::ty::markers::SpannerProto> IntoSpanner
    for AsProtoEnum<T>
{
    type SpannerType = Self;

    fn into_value(self) -> Value {
        let int_repr: i32 = self.0.into();

        // between the tag + value, this can be up to 16 bytes for a 64 bit value, so just use 16 to
        // be safe.
        let mut buf = [0_u8; 16];
        let mut buf_slice = &mut buf[..];

        prost::Message::encode(&int_repr, &mut buf_slice).expect("should be a big enough buffer");

        let remaining = buf_slice.len();
        let size = buf.len() - remaining;

        encode(&buf[..size]).into_value()
    }
}

// ==================== Enum =======================

pub struct Enum<T>(pub T);

impl<T> SpannerType for Enum<T>
where
    T: Into<i32>,
    T: TryFrom<i32>,
    <T as TryFrom<i32>>::Error: std::error::Error + Send + Sync + 'static,
    T: crate::ty::markers::SpannerEnum,
{
    type Type = crate::ty::markers::ProtoEnum<T>;
    type Nullable = typenum::False;
}

impl<T> IntoSpanner for Enum<T>
where
    T: Into<i32>,
    T: TryFrom<i32>,
    <T as TryFrom<i32>>::Error: std::error::Error + Send + Sync + 'static,
    T: crate::ty::markers::SpannerEnum,
{
    type SpannerType = Self;
    fn into_value(self) -> Value {
        <i32 as IntoSpanner>::into_value(self.0.into())
    }
}

impl<T> FromSpanner for Enum<T>
where
    T: Into<i32>,
    T: TryFrom<i32>,
    <T as TryFrom<i32>>::Error: std::error::Error + Send + Sync + 'static,
    T: crate::ty::markers::SpannerEnum,
{
    type SpannerType = Self;

    fn from_value(value: Value) -> Result<Self, ConvertError> {
        pub use Kind::*;

        let enum_int_repr = match &value.0 {
            NumberValue(num) => num.round() as i32,
            StringValue(string) => match string.parse::<i32>() {
                Ok(num) => num,
                Err(err) => return Err(FromError::from_value_and_error::<Self>(value, err).into()),
            },
            _ => return Err(FromError::from_value::<Self>(value).into()),
        };

        match T::try_from(enum_int_repr) {
            Ok(value) => Ok(Self(value)),
            Err(error) => Err(FromError::from_value_and_error::<Self>(value, error).into()),
        }
    }
}
