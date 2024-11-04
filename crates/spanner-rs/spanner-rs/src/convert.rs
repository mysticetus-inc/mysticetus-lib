use std::borrow::Cow;
use std::sync::Arc;

use bytes::Bytes;
use protos::protobuf::value::Kind;
use timestamp::{Date, Timestamp};

use crate::error::{ConvertError, FromError, IntoError};
use crate::ty::{Scalar, SpannerType, Type};
use crate::value::{EncodedArray, EncodedValue};
use crate::{Field, Value};

/// Trait for simple types that can be converted directly to a [`Value`], infallibly.
///
/// All types that are meant to convert to a spanner column data types should implement this.
///
/// If a type has a fallible conversion to a [`Value`], [`SpannerEncode`] should be implemented
/// instead.
pub trait IntoSpanner: SpannerType {
    /// Converts 'self' into a [`Value`]
    fn into_value(self) -> Value;
}

/// Trait to parse a value from an encoded Spanner [`Value`].
pub trait FromSpanner: SpannerEncode + Sized {
    fn from_value(value: Value) -> Result<Self, ConvertError>;

    fn from_field_and_value(field: &Field, value: Value) -> Result<Self, ConvertError> {
        match Self::from_value(value) {
            Ok(value) => Ok(value),
            Err(err) => Err(err.column(field.name.clone())),
        }
    }
}

/// A (sort of) parent trait to [`IntoSpanner`] for types with fallible/complex conversions
/// into a [`Value`], namely nested types like structs, arrays and json.
///
/// Simple types with infallible conversions should implement [`IntoSpanner`], since there's
/// a blanket implementation of [`SpannerEncode`] for all [`IntoSpanner`] types.
pub trait SpannerEncode {
    /// A marker type to indicate the data type of this type in Spanner.
    ///
    /// This is an associated type rather than a trait bound, to let specific types
    /// define their own Spanner representation rather than falling under a possibly
    /// incorrect blanket implementation (the motivating type for this was [`Vec<u8>`]
    /// clearly representing bytes, but for any other element type, [`Vec<T>`] should
    /// be an array).
    ///
    /// We can't assume the <code><Self::Encoded as IntoSpanner>::SpannerType</code>
    /// marker is the correct type to use as a default, since the encoded type is
    /// likely to have nothing to do with the target spanner type.
    ///
    /// For example, Spanner JSON values need to be encoded as valid JSON strings
    /// for transport. Because of this, the [`Json<T>`] implementation
    /// has <code>[`Encoded`] = [`String`]</code>, which would indicate that the
    /// target spanner type is just a regular string, which is incorrect.
    ///
    /// [`Encoded`]: SpannerEncode::Encoded
    type SpannerType: SpannerType;

    /// The resulting encoded type, which need's be convertable into a Spanner [`Value`].
    type Encoded: IntoSpanner;

    /// The error type for fallible encodings. Generic to allow for non-[`TypeError`]'s,
    /// including [`Infallible`]/[`!`] if the encoding is infallible.
    ///
    /// [`Infallible`]: std::convert::Infallible
    /// [`!`]: https://doc.rust-lang.org/std/primitive.never.html
    type Error: Into<ConvertError>;

    /// Encode 'self'.
    fn encode(self) -> Result<Self::Encoded, Self::Error>;

    #[inline]
    fn encode_to_value(self) -> Result<Value, Self::Error>
    where
        Self: Sized,
        Self::Encoded: IntoSpanner,
    {
        match self.encode() {
            Ok(encoded) => Ok(encoded.into_value()),
            Err(err) => Err(err),
        }
    }
}

/// Blanket implementation over all [`IntoSpanner`] types.
impl<T> SpannerEncode for T
where
    T: IntoSpanner,
{
    type SpannerType = T;

    type Encoded = Self;

    type Error = std::convert::Infallible;

    #[inline(always)]
    fn encode(self) -> Result<Self::Encoded, std::convert::Infallible> {
        Ok(self)
    }
}

// -------------------- serde_json::Value ------------------ //

impl SpannerEncode for serde_json::Value {
    type SpannerType = crate::with::Json<Self>;
    type Error = IntoError;
    type Encoded = String;

    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        crate::with::Json(&self).encode()
    }
}

impl SpannerEncode for &serde_json::Value {
    type SpannerType = crate::with::Json<Self>;
    type Error = IntoError;
    type Encoded = String;

    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        crate::with::Json(self).encode()
    }
}

impl FromSpanner for serde_json::Value {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        crate::with::Json::from_value(value).map(|j| j.0)
    }
}

impl<K, V> SpannerEncode for std::collections::BTreeMap<K, V>
where
    K: serde::Serialize,
    V: serde::Serialize,
{
    type Error = IntoError;
    type Encoded = String;
    type SpannerType = crate::with::Json<Self>;

    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        crate::with::Json(self).encode()
    }
}

// -------------- String/&str ------------------ //

macro_rules! impl_str_into {
    ($($s:ty $(: $conv_fn:ident)? ),* $(,)?) => {
        $(
            impl IntoSpanner for $s {
                // type SpannerType = String;

                #[inline]
                fn into_value(self) -> Value {
                    Value(protos::protobuf::value::Kind::StringValue(String::from(
                        self $(.$conv_fn())?
                    )))
                }
            }
        )*
    };
}

impl_str_into! {
    &str,
    String,
    Cow<'_, str>: into_owned,
    Box<str>: into_string,
    std::rc::Rc<str>: as_ref,
    std::sync::Arc<str>: as_ref,
}

impl FromSpanner for String {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value.into_string::<Self>().map_err(ConvertError::from)
    }
}

impl<'a> FromSpanner for Cow<'a, str> {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value
            .into_string::<Self>()
            .map(Cow::Owned)
            .map_err(ConvertError::from)
    }
}

impl FromSpanner for std::sync::Arc<str> {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value
            .into_string::<Self>()
            .map(std::sync::Arc::from)
            .map_err(ConvertError::from)
    }
}

impl FromSpanner for std::rc::Rc<str> {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value
            .into_string::<Self>()
            .map(std::rc::Rc::from)
            .map_err(ConvertError::from)
    }
}

impl FromSpanner for Box<str> {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value
            .into_string::<Self>()
            .map(String::into_boxed_str)
            .map_err(ConvertError::from)
    }
}

// -------------- boolean ------------------ //

impl IntoSpanner for bool {
    #[inline]
    fn into_value(self) -> Value {
        Value(Kind::BoolValue(self))
    }
}

impl FromSpanner for bool {
    #[inline]
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value.into_bool::<Self>().map_err(ConvertError::from)
    }
}

// -------------- regular + nonzero integers ------------------ //

macro_rules! impl_ints {
    ($($non_zero_int:ty => $int:ty),* $(,)?) => {
        $(

            // ------------ regular impls ----------------- //
            impl IntoSpanner for $int {
                fn into_value(self) -> Value {
                    itoa::Buffer::new()
                        .format(self)
                        .into_value()
                }
            }

            impl FromSpanner for $int {
                fn from_value(value: Value) -> Result<Self, ConvertError> {
                    match value.0 {
                        Kind::StringValue(s) => match s.parse::<Self>() {
                            Ok(parsed) => Ok(parsed),
                            Err(err) => Err(FromError::from_value_and_error::<Self>(s, err).into())
                        },
                        Kind::NumberValue(n) => Ok(n as $int),
                        other => Err(FromError::from_value::<Self>(other).into())
                    }
                }
            }

            // ------------ nonzero impl ----------------- //
            impl IntoSpanner for $non_zero_int {
                fn into_value(self) -> Value {
                    self.get().into_value()
                }
            }

            impl FromSpanner for $non_zero_int {
                fn from_value(value: Value) -> Result<Self, ConvertError> {
                    let int = <$int>::from_value(value)?;

                    match Self::new(int) {
                        Some(non_zero) => Ok(non_zero),
                        None => Err(FromError::from_anyhow::<Self>(anyhow::anyhow!("invalid value, must be non-zero")).into()),
                    }
                }
            }
        )*
    };
}

impl_ints! {
    std::num::NonZeroI8 => i8,
    std::num::NonZeroI16 => i16,
    std::num::NonZeroI32 => i32,
    std::num::NonZeroI64 => i64,
    std::num::NonZeroI128 => i128,
    std::num::NonZeroIsize => isize,
    std::num::NonZeroU8 => u8,
    std::num::NonZeroU16 => u16,
    std::num::NonZeroU32 => u32,
    std::num::NonZeroU64 => u64,
    std::num::NonZeroU128 => u128,
    std::num::NonZeroUsize => usize,
}

// -------------- f32/f64 ------------------ //

impl IntoSpanner for f64 {
    #[inline]
    fn into_value(self) -> Value {
        Value::from(self)
    }
}

impl IntoSpanner for f32 {
    #[inline]
    fn into_value(self) -> Value {
        Value::from(self as f64)
    }
}

impl FromSpanner for f64 {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        match value.0 {
            Kind::StringValue(s) => match s.as_str() {
                "NaN" => Ok(Self::NAN),
                "Infinity" => Ok(Self::INFINITY),
                "-Infinity" => Ok(Self::NEG_INFINITY),
                _ => Err(FromError::from_value::<Self>(s).into()),
            },
            Kind::NumberValue(n) => Ok(n),
            other => Err(FromError::from_value::<Self>(other).into()),
        }
    }
}

impl FromSpanner for f32 {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        f64::from_value(value).map(|double| double as f32)
    }
}

// -------------- char ------------------ //

impl SpannerType for char {
    const TYPE: &'static Type = &Type::Scalar(Scalar::String);
    const NULLABLE: bool = false;
}

impl IntoSpanner for char {
    #[inline]
    fn into_value(self) -> Value {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf).into_value()
    }
}

impl FromSpanner for char {
    #[inline]
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = value.into_string::<Self>()?;

        let mut chars = s.chars();

        let ch = match chars.next() {
            Some(ch) => ch,
            None => {
                return Err(FromError::from_value_and_anyhow::<Self>(
                    s,
                    anyhow::anyhow!("cannot convert empty string to a char"),
                )
                .into());
            }
        };

        if chars.next().is_some() {
            return Err(FromError::from_value_and_anyhow::<Self>(
                s,
                anyhow::anyhow!("expected a single character"),
            )
            .into());
        }

        Ok(ch)
    }
}

// -------------- Vec<I> ------------------ //

impl<T> SpannerEncode for Vec<T>
where
    T: SpannerEncode,
{
    type SpannerType = EncodedArray<T::Encoded>;

    type Encoded = EncodedArray<T::Encoded>;

    type Error = T::Error;

    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        EncodedArray::encode_from(self)
    }
}

impl<T> FromSpanner for Vec<T>
where
    T: FromSpanner,
{
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        value
            .into_array::<Self>()?
            .values
            .into_iter()
            .map(|v| T::from_value(Value::from_protobuf(v)))
            .collect::<Result<Vec<T>, ConvertError>>()
    }
}

macro_rules! impl_box_arc_slice_encode {
    ($ptr_ty:ident) => {
        impl<T> SpannerEncode for $ptr_ty<[T]>
        where
            T: SpannerEncode + Clone,
        {
            type SpannerType = EncodedArray<T::Encoded>;

            type Encoded = EncodedArray<T::Encoded>;

            type Error = T::Error;

            fn encode(self) -> Result<Self::Encoded, Self::Error> {
                EncodedArray::encode_from(self.iter().cloned())
            }
        }

        impl<T> FromSpanner for $ptr_ty<[T]>
        where
            T: FromSpanner + Clone,
        {
            fn from_value(value: Value) -> Result<Self, ConvertError> {
                let boxed_slice = value
                    .into_array::<Self>()?
                    .values
                    .into_iter()
                    .map(|v| T::from_value(Value::from_protobuf(v)))
                    .collect::<Result<Vec<T>, ConvertError>>()?
                    .into_boxed_slice();

                Ok(Self::from(boxed_slice))
            }
        }
    };
}

impl_box_arc_slice_encode!(Box);
impl_box_arc_slice_encode!(Arc);

// -------------- Option<T> ------------------ //

impl<T> SpannerEncode for Option<T>
where
    T: SpannerEncode,
{
    type SpannerType = T::SpannerType;
    type Error = T::Error;
    type Encoded = EncodedValue<T::Encoded>;

    fn encode(self) -> Result<Self::Encoded, Self::Error> {
        match self.map(T::encode) {
            Some(result) => result.map(EncodedValue::encode),
            None => Ok(EncodedValue::NULL),
        }
    }

    // overwrite the default impl, since this can be done much more efficently than even the
    // 'encode' impl.
    fn encode_to_value(self) -> Result<Value, T::Error> {
        match self {
            Some(inner) => inner.encode_to_value(),
            None => Ok(Value::NULL),
        }
    }
}

impl<T> FromSpanner for Option<T>
where
    T: FromSpanner,
{
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        match &value.0 {
            Kind::NullValue(_) => Ok(None),
            _ => T::from_value(value).map(Some),
        }
    }
}

// ---------------- &[u8]-like types ------------------- //

impl IntoSpanner for &[u8] {
    fn into_value(self) -> Value {
        crate::with::AsBytes(self).into_value()
    }
}

impl IntoSpanner for Bytes {
    fn into_value(self) -> Value {
        crate::with::AsBytes(self.as_ref()).into_value()
    }
}

impl FromSpanner for Bytes {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        crate::with::AsBytes::from_value(value).map(|b| b.0)
    }
}

// -------------- Date ------------------ //

impl IntoSpanner for Date {
    fn into_value(self) -> Value {
        let mut dst = String::new();
        self.append_to_string(&mut dst);
        dst.into_value()
    }
}

impl FromSpanner for Date {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = value.into_string::<Self>()?;

        match s.parse() {
            Ok(date) => Ok(date),
            Err(err) => Err(FromError::from_value_and_error::<Self>(s, err).into()),
        }
    }
}

// -------------- Timestamp ------------------ //
impl IntoSpanner for Timestamp {
    fn into_value(self) -> Value {
        self.as_iso8601().into_value()
    }
}

impl FromSpanner for Timestamp {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = value.into_string::<Self>()?;

        match s.parse() {
            Ok(ts) => Ok(ts),
            Err(err) => Err(FromError::from_value_and_error::<Self>(s, err).into()),
        }
    }
}

// -------------- CommitTimestamp ------------------ //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitTimestamp<T = Option<Timestamp>>(pub T);

impl SpannerType for CommitTimestamp<Option<Timestamp>> {
    const TYPE: &'static Type = &Type::Scalar(Scalar::Timestamp);
    const NULLABLE: bool = true;
}

impl SpannerType for CommitTimestamp<()> {
    const TYPE: &'static Type = &Type::Scalar(Scalar::Timestamp);
    const NULLABLE: bool = false;
}

impl<T> CommitTimestamp<T> {
    const COMMIT_STR: &'static str = "spanner.commit_timestamp()";
}

impl IntoSpanner for CommitTimestamp<Option<Timestamp>> {
    fn into_value(self) -> Value {
        match self.0 {
            None => Self::COMMIT_STR.into_value(),
            Some(t) => t.into_value(),
        }
    }
}

impl IntoSpanner for CommitTimestamp<()> {
    fn into_value(self) -> Value {
        Self::COMMIT_STR.into_value()
    }
}

impl FromSpanner for CommitTimestamp<Option<Timestamp>> {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        Option::from_value(value).map(CommitTimestamp)
    }
}

impl FromSpanner for CommitTimestamp<()> {
    fn from_value(_value: Value) -> Result<Self, ConvertError> {
        Ok(Self(()))
    }
}

// ------------------ Uuid --------------------- //
impl SpannerType for uuid::Uuid {
    const TYPE: &'static Type = &Type::Scalar(Scalar::String);
    const NULLABLE: bool = false;
}

impl IntoSpanner for uuid::Uuid {
    fn into_value(self) -> Value {
        let mut buf = [0; 36];

        self.hyphenated().encode_lower(&mut buf).into_value()
    }
}

impl FromSpanner for uuid::Uuid {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        let s = value.into_string::<Self>()?;

        match s.parse::<Self>() {
            Ok(uuid) => Ok(uuid),
            Err(err) => Err(FromError::from_value_and_error::<Self>(s, err).into()),
        }
    }
}

// TODO hacky workaround to deal with options

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NullableString(pub Option<String>);

impl SpannerType for NullableString {
    const TYPE: &'static Type = &Type::STRING;
    const NULLABLE: bool = true;
}

impl IntoSpanner for NullableString {
    fn into_value(self) -> Value {
        self.0.map(IntoSpanner::into_value).unwrap_or(Value::NULL)
    }
}

impl FromSpanner for NullableString {
    fn from_value(value: Value) -> Result<Self, ConvertError> {
        Option::<String>::from_value(value).map(Self)
    }
}

impl From<Option<String>> for NullableString {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

impl From<NullableString> for Option<String> {
    fn from(value: NullableString) -> Self {
        value.0
    }
}
