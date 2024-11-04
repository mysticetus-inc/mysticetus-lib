use std::collections::HashMap;
use std::marker::PhantomData;

use serde::de;

use super::{Array, Map};

pub(super) struct ValueVisitor;

struct ExpectedErr<E>(E);

impl<E: std::fmt::Display> de::Expected for ExpectedErr<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

macro_rules! forward_int_impls {
    (
        $(
            $name:ident($arg_type:ty) -> $unexpected:ident
        ),* $(,)?
    ) => {
        $(
            #[inline]
            fn $name<E>(self, v: $arg_type) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let i: i64 = v
                    .try_into()
                    .map_err(|err| E::invalid_value(de::Unexpected::$unexpected(v as _), &ExpectedErr(err)))?;
                self.visit_i64(i)
            }
        )*
    };
    (@IMPL_ALL) => {
        forward_int_impls! {
            visit_i8(i8) -> Signed,
            visit_i16(i16) -> Signed,
            visit_i32(i32) -> Signed,
            visit_i128(i128) -> Signed,

            visit_u8(u8) -> Unsigned,
            visit_u16(u16) -> Unsigned,
            visit_u32(u32) -> Unsigned,
            visit_u64(u64) -> Unsigned,
            visit_u128(u128) -> Unsigned,
        }
    };
}

macro_rules! impl_defferred_fns {
    () => {
        forward_int_impls!(@IMPL_ALL);

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_byte_buf(v.to_vec())
        }


        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {

            self.visit_string(v.to_owned())
        }

        fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let mut b = [0; 4];
            self.visit_string(v.encode_utf8(&mut b).to_owned())
        }

        fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_f64(v as f64)
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_unit()
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }
    };
}

impl<'de> de::Visitor<'de> for ValueVisitor {
    type Value = super::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid firestore value")
    }

    impl_defferred_fns!();

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(super::Value::Bool(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(super::Value::Bytes(v.into()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(super::Value::String(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(super::Value::Double(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(super::Value::Integer(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(super::Value::Null)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        visit_seq_inner(seq).map(|values| super::Value::Array(Array::from(values)))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        visit_map_inner(map).map(|fields| super::Value::Map(Map::from(fields)))
    }
}

pub(super) struct RawValueVisitor;

impl<'de> de::DeserializeSeed<'de> for RawValueVisitor {
    type Value = protos::firestore::value::ValueType;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> de::Visitor<'de> for RawValueVisitor {
    type Value = protos::firestore::value::ValueType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid firestore value")
    }

    impl_defferred_fns!();

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(protos::firestore::value::ValueType::BooleanValue(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(protos::firestore::value::ValueType::BytesValue(v.into()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(protos::firestore::value::ValueType::DoubleValue(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(protos::firestore::value::ValueType::IntegerValue(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(protos::firestore::value::ValueType::StringValue(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(protos::firestore::value::ValueType::NullValue(
            protos::protobuf::NullValue::NullValue as i32,
        ))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        visit_map_inner(map).map(|fields| {
            protos::firestore::value::ValueType::MapValue(protos::firestore::MapValue { fields })
        })
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        visit_seq_inner(seq).map(|values| {
            protos::firestore::value::ValueType::ArrayValue(protos::firestore::ArrayValue {
                values,
            })
        })
    }
}

pub(super) fn visit_seq_inner<'de, A>(mut seq: A) -> Result<Vec<protos::firestore::Value>, A::Error>
where
    A: de::SeqAccess<'de>,
{
    let mut dst = Vec::with_capacity(seq.size_hint().unwrap_or(32));

    while let Some(val) = seq.next_element_seed(RawValueVisitor)? {
        dst.push(protos::firestore::Value {
            value_type: Some(val),
        });
    }

    Ok(dst)
}

pub(super) fn visit_map_inner<'de, A>(
    mut map: A,
) -> Result<HashMap<String, protos::firestore::Value>, A::Error>
where
    A: de::MapAccess<'de>,
{
    let mut dst = HashMap::with_capacity(map.size_hint().unwrap_or(32));

    while let Some((key, val)) = map.next_entry_seed(PhantomData, RawValueVisitor)? {
        dst.insert(
            key,
            protos::firestore::Value {
                value_type: Some(val),
            },
        );
    }

    Ok(dst)
}
