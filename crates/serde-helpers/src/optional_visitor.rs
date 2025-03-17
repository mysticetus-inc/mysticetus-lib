use std::fmt;

use serde::de;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OptionalVisitor<V>(pub V);

impl<V> From<V> for OptionalVisitor<V> {
    fn from(value: V) -> Self {
        Self(value)
    }
}

macro_rules! impl_passthrough_fn {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<E>(self, v: $arg_ty) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.0.$fn_name(v).map(Some)
            }
        )*
    };
}

impl<'de, V> de::Visitor<'de> for OptionalVisitor<V>
where
    V: de::DeserializeSeed<'de> + de::Visitor<'de, Value = <V as de::DeserializeSeed<'de>>::Value>,
{
    type Value = Option<<V as de::Visitor<'de>>::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
        self.0.expecting(formatter)?;
        formatter.write_str(" (optional)")
    }

    //  ---------- option specific methods ------------
    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.0.deserialize(deserializer).map(Some)
    }

    //  ---------- passthrough methods ------------

    impl_passthrough_fn! {
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_i128(i128),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_u128(u128),

        visit_f32(f32),
        visit_f64(f64),

        visit_char(char),
        visit_bool(bool),

        visit_str(&str),
        visit_borrowed_str(&'de str),
        visit_string(String),

        visit_bytes(&[u8]),
        visit_borrowed_bytes(&'de [u8]),
        visit_byte_buf(Vec<u8>),
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.0.visit_seq(seq).map(Some)
    }

    #[inline]
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.0.visit_map(map).map(Some)
    }

    #[inline]
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.0.visit_enum(data).map(Some)
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.0.visit_newtype_struct(deserializer).map(Some)
    }
}

impl<'de, V> de::DeserializeSeed<'de> for OptionalVisitor<V>
where
    V: de::DeserializeSeed<'de> + de::Visitor<'de, Value = <V as de::DeserializeSeed<'de>>::Value>,
{
    type Value = Option<<V as de::Visitor<'de>>::Value>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}
