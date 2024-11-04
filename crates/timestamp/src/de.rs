//! Timestamp deserialization methods + impl
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Unexpected};
use serde::{Deserialize, forward_to_deserialize_any, serde_if_integer128};

use crate::error::{ConvertError, Error, Num};
use crate::{Timestamp, Unit};

impl Timestamp {
    /// Deserializes a timestamp, expecting to find a number in the given unit, or a datetime
    /// string.
    fn deserialize_from_unit<'de, D, P>(deserializer: D, parser: P) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
        P: ParseUnit,
    {
        deserializer.deserialize_any(TimestampVisitor::new(parser))
    }

    /// Deserializes a timestamp, expecting to find a number in the given unit, or a datetime
    /// string.
    fn deserialize_from_unit_opt<'de, D, P>(
        deserializer: D,
        parser: P,
    ) -> Result<Option<Self>, D::Error>
    where
        D: de::Deserializer<'de>,
        P: ParseUnit,
    {
        deserializer.deserialize_any(OptionalVisitor(TimestampVisitor::new(parser)))
    }

    /// Deserializes a timestamp, expecting to find a number in seconds, or a datetime string.
    ///
    /// The default [`Deserialize`] impl calls this under the hood.
    pub fn deserialize_from_seconds<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit(deserializer, Seconds)
    }

    /// Deserializes a timestamp, expecting to find a number in seconds or milliseconds, or a
    /// datetime string.
    ///
    /// The default [`Deserialize`] impl calls this under the hood.
    pub fn deserialize_fuzzy<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit(deserializer, SecondsOrMillis)
    }

    /// Deserializes a timestamp, expecting to find a number in milliseconds, or a datetime string.
    ///
    /// The default [`Deserialize`] impl calls this under the hood.
    pub fn deserialize_from_millis<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit(deserializer, Millis)
    }

    /// Deserializes a timestamp, expecting to find a number in microseconds, or a datetime string.
    ///
    /// The default [`Deserialize`] impl calls this under the hood.
    pub fn deserialize_from_micros<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit(deserializer, Unit::Micros)
    }

    /// Deserializes a timestamp, expecting to find a number in nanoseconds, or a datetime string.
    ///
    /// The default [`Deserialize`] impl calls this under the hood.
    pub fn deserialize_from_nanos<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit(deserializer, Unit::Nanos)
    }

    /// Deserializes an optional timestamp, expecting a number in seconds, or a datetime string.
    pub fn deserialize_from_seconds_opt<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit_opt(deserializer, Seconds)
    }

    /// Deserializes an optional timestamp, expecting a number in milliseconds,
    /// or a datetime string.
    pub fn deserialize_from_millis_opt<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit_opt(deserializer, Millis)
    }

    /// Deserializes an optional timestamp, expecting a number in microseconds,
    /// or a datetime string.
    pub fn deserialize_from_micros_opt<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit_opt(deserializer, Unit::Micros)
    }

    /// Deserializes an optional timestamp, expecting a number in nanoseconds,
    /// or a datetime string.
    pub fn deserialize_from_nanos_opt<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Self::deserialize_from_unit_opt(deserializer, Unit::Nanos)
    }

    // -------------------------- IntoDeserializer helpers -------------------------- //

    /// Generic deserializer builder. Creates a deserializer that will attempt to deserialize
    /// as the specified unit, or as a datetime string if [`deserialize_str`] (or another
    /// deserialize string variant) is called.
    ///
    /// [`deserialize_str`]: [`Deserializer::deserialize_str`]
    pub fn into_unit_deserializer<'a>(self, unit: Unit) -> TimestampDeserializer<'a> {
        TimestampDeserializer::new(self, unit)
    }

    /// Creates a deserializer that will attempt to deserialize into seconds, or as a datetime
    /// string if [`deserialize_str`] (or another deserialize string variant) is called.
    ///
    /// The default impl of [`IntoDeserializer`] uses this variant under the hood.
    ///
    /// [`deserialize_str`]: [`Deserializer::deserialize_str`]
    /// [`IntoDeserializer`]: [`serde::de::IntoDeserializer`]
    pub fn into_second_deserializer<'a>(self) -> TimestampDeserializer<'a> {
        self.into_unit_deserializer(Unit::Seconds)
    }

    /// Creates a deserializer that will attempt to deserialize into milliseconds, or as a
    /// datetime string if [`deserialize_str`] (or another deserialize string variant) is called.
    ///
    /// [`deserialize_str`]: [`Deserializer::deserialize_str`]
    pub fn into_millis_deserializer<'a>(self) -> TimestampDeserializer<'a> {
        self.into_unit_deserializer(Unit::Millis)
    }

    /// Creates a deserializer that will attempt to deserialize into nanoseconds, or as a
    /// datetime string if [`deserialize_str`] (or another deserialize string variant) is called.
    ///
    /// [`deserialize_str`]: [`Deserializer::deserialize_str`]
    pub fn into_nanos_deserializer<'a>(self) -> TimestampDeserializer<'a> {
        self.into_unit_deserializer(Unit::Nanos)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Timestamp::deserialize_from_seconds(deserializer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TimestampVisitor<Parser>(Parser);

trait ParseUnit {
    fn expected_unit(&self) -> &str;

    // fn handle_str(&self, s: &str) -> Result<Timestamp, error::Error> {
    //    s.parse::<Timestamp>()
    // }

    fn handle_f64(&self, f: f64) -> Result<Timestamp, ConvertError>;

    fn handle_i64(&self, i: i64) -> Result<Timestamp, ConvertError>;

    fn handle_u64(&self, u: u64) -> Result<Timestamp, ConvertError> {
        u.try_into()
            .map_err(|_| (Num::U64, Num::I64).into())
            .and_then(|i| self.handle_i64(i))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Seconds;

impl ParseUnit for Seconds {
    fn expected_unit(&self) -> &str {
        "seconds"
    }

    fn handle_i64(&self, i: i64) -> Result<Timestamp, ConvertError> {
        Timestamp::from_seconds_checked(i)
    }

    fn handle_f64(&self, f: f64) -> Result<Timestamp, ConvertError> {
        Timestamp::from_seconds_f64_checked(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Millis;

impl ParseUnit for Millis {
    fn expected_unit(&self) -> &str {
        "milliseconds"
    }

    fn handle_i64(&self, i: i64) -> Result<Timestamp, ConvertError> {
        Timestamp::from_millis_checked(i)
    }

    fn handle_f64(&self, f: f64) -> Result<Timestamp, ConvertError> {
        Timestamp::from_millis_f64_checked(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SecondsOrMillis;

impl ParseUnit for SecondsOrMillis {
    fn expected_unit(&self) -> &str {
        "seconds or milliseconds"
    }

    fn handle_i64(&self, i: i64) -> Result<Timestamp, ConvertError> {
        Seconds.handle_i64(i).or_else(|_| Millis.handle_i64(i))
    }

    fn handle_f64(&self, f: f64) -> Result<Timestamp, ConvertError> {
        Seconds.handle_f64(f).or_else(|_| Millis.handle_f64(f))
    }
}

impl ParseUnit for Unit {
    fn expected_unit(&self) -> &str {
        self.as_str()
    }

    fn handle_i64(&self, i: i64) -> Result<Timestamp, ConvertError> {
        match self {
            Unit::Seconds => Seconds.handle_i64(i),
            Unit::Millis => Millis.handle_i64(i),
            Unit::Micros => Timestamp::from_micros_checked(i),
            Unit::Nanos => Ok(Timestamp::from_nanos(i)),
        }
    }

    fn handle_f64(&self, f: f64) -> Result<Timestamp, ConvertError> {
        match self {
            Unit::Seconds => Seconds.handle_f64(f),
            Unit::Millis => Millis.handle_f64(f),
            Unit::Micros => Timestamp::from_micros_f64_checked(f),
            Unit::Nanos => Ok(Timestamp::from_nanos_f64(f)),
        }
    }
}

impl<Parser: ParseUnit> TimestampVisitor<Parser> {
    fn new(parser: Parser) -> Self {
        Self(parser)
    }
}

impl<'de, Parser: ParseUnit> de::Visitor<'de> for TimestampVisitor<Parser> {
    type Value = Timestamp;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a unix timestamp in {}, or a datetime string",
            self.0.expected_unit()
        )
    }

    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        string
            .parse::<Timestamp>()
            .map_err(|e| e.into_serde(Unexpected::Str(string)))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, float: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0
            .handle_f64(float)
            .map_err(|e| e.into_serde(Unexpected::Float(float)))
    }

    fn visit_i64<E>(self, int: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0
            .handle_i64(int)
            .map_err(|e| e.into_serde(Unexpected::Signed(int)))
    }

    fn visit_i32<E>(self, int: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(int as i64)
    }

    fn visit_i16<E>(self, int: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(int as i64)
    }

    fn visit_i8<E>(self, int: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(int as i64)
    }

    fn visit_u64<E>(self, uint: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0
            .handle_u64(uint)
            .map_err(|e| e.into_serde(Unexpected::Unsigned(uint)))
    }

    fn visit_u32<E>(self, uint: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(uint as u64)
    }

    fn visit_u16<E>(self, uint: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(uint as u64)
    }

    fn visit_u8<E>(self, uint: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(uint as u64)
    }

    serde::serde_if_integer128! {
        fn visit_i128<E>(self, int: i128) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            let downcasted: i64 = int.try_into()
                .map_err(|_| ConvertError::OutOfRange.into_serde(
                    Unexpected::Other("128 bit signed int, out of range of a 64 bit int")
                ))?;


            self.visit_i64(downcasted)
        }

        fn visit_u128<E>(self, int: u128) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            let downcasted: u64 = int.try_into()
                .map_err(|_| ConvertError::OutOfRange.into_serde(
                    Unexpected::Other("128 bit uint, out of range of a 64 bit uint")
                ))?;

            self.visit_u64(downcasted)
        }
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> de::IntoDeserializer<'de, Error> for Timestamp {
    type Deserializer = TimestampDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.into_second_deserializer()
    }
}

/// A [`Deserializer`] containing a [`Timestamp`] and [`Unit`], which can be used to
/// deserialize into specific formats.
///
/// [`Deserializer`]: [`de::Deserializer`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampDeserializer<'de> {
    ts: Timestamp,
    unit: Unit,
    _marker: PhantomData<&'de ()>,
}

impl<'a> TimestampDeserializer<'a> {
    /// Builds a new [`TimestampDeserializer`] with a given [`Timestamp`]. When deserialized,
    /// the specified [`Unit`] will determine what unit numeric types are deserialized as.
    pub fn new(ts: Timestamp, unit: Unit) -> Self {
        Self {
            ts,
            unit,
            _marker: PhantomData,
        }
    }

    /// Shortcut to the [`Timestamp::to_unit_signed`] method.
    fn to_unit_signed(self) -> i64 {
        self.ts.to_unit_signed(self.unit)
    }

    /// Shortcut to the [`Timestamp::to_unit_unsigned`] method.
    fn to_unit_unsigned(self) -> Result<u64, ConvertError> {
        self.ts.to_unit_unsigned(self.unit)
    }

    /// Shortcut to the [`Timestamp::to_unit_f64`] method.
    fn to_unit_f64(self) -> f64 {
        self.ts.to_unit_f64(self.unit)
    }
}

impl<'de> de::Deserializer<'de> for TimestampDeserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 u8 u16 char bytes byte_buf unit unit_struct map
        struct enum ignored_any identifier
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.ts.as_seconds_f64())
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let downcasted: i32 = self
            .to_unit_signed()
            .try_into()
            .map_err(|_| ConvertError::from((Num::I64, Num::I32)))?;

        visitor.visit_i32(downcasted)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.to_unit_signed())
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let num: u64 = self.to_unit_unsigned()?;

        let downcasted: u32 = num
            .try_into()
            .map_err(|_| ConvertError::from((Num::U64, Num::U32)))?;

        visitor.visit_u32(downcasted)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let num: u64 = self.to_unit_unsigned()?;
        visitor.visit_u64(num)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.to_unit_f64() as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.to_unit_f64())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // we always have to allocate here, so pass this off to [`visit_string`]
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let iso_str = self.ts.as_iso8601();
        visitor.visit_string(iso_str)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // If the variant has 1 element, assume we're just wrapping a timestamp.
        if len != 1 {
            return Err(Error::Custom(
                "cannot coerce a timestamp across multiple tuple elements".to_owned(),
            ));
        }

        visitor.visit_seq(TimestampSeqAccess(Some(self)))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(TimestampSeqAccess(Some(self)))
    }

    serde_if_integer128! {
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_i128(self.to_unit_signed() as i128)
        }

        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let num: u64 = self.to_unit_unsigned()?;
            visitor.visit_u128(num as u128)
        }
    }
}

struct TimestampSeqAccess<'de>(Option<TimestampDeserializer<'de>>);

impl<'de> de::SeqAccess<'de> for TimestampSeqAccess<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        // Take the option, that way we only yeild the initial timestamp
        match self.0.take() {
            Some(deser) => seed.deserialize(deser).map(Some),
            None => Ok(None),
        }
    }
}

/// Helper visitor for optional deserialization
pub(crate) struct OptionalVisitor<V>(pub(crate) V);

macro_rules! defer_to_inner {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            fn $fn_name<E>(self, arg: $arg_ty) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                self.0.$fn_name(arg).map(Some)
            }
        )*
    };
}

impl<'de, V> de::Visitor<'de> for OptionalVisitor<V>
where
    V: de::Visitor<'de>,
{
    type Value = Option<<V as de::Visitor<'de>>::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.expecting(formatter)?;
        formatter.write_str(" (optional)")
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.visit_some(deserializer).map(Some)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.0.visit_enum(data).map(Some)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.visit_newtype_struct(deserializer).map(Some)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.0.visit_seq(seq).map(Some)
    }

    fn visit_map<Map>(self, map: Map) -> Result<Self::Value, Map::Error>
    where
        Map: de::MapAccess<'de>,
    {
        self.0.visit_map(map).map(Some)
    }

    defer_to_inner! {
        visit_bool(bool),
        visit_borrowed_bytes(&'de [u8]),
        visit_borrowed_str(&'de str),
        visit_byte_buf(Vec<u8>),
        visit_bytes(&[u8]),
        visit_char(char),
        visit_f32(f32),
        visit_f64(f64),
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_str(&str),
        visit_string(String),
    }

    serde_if_integer128! {
        defer_to_inner! {
            visit_i128(i128),
            visit_u128(u128),
        }
    }
}
