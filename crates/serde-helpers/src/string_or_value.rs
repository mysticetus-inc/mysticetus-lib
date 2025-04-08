use std::fmt;
use std::str::FromStr;

use serde::de::{self, Deserialize, IntoDeserializer};

use super::never_visitor::NeverVisitor;
use crate::string_dst::{ParsableDst, StringDst};

#[derive(Debug)]
pub struct StringOrValue<T, Dst, S = NeverVisitor<T>> {
    string_dst: Dst,
    is_some: bool,
    value: Option<T>,
    seed: Option<S>,
}

impl<T, Dst> StringOrValue<T, Dst> {
    pub fn new(string_dst: Dst) -> Self {
        Self {
            string_dst,
            is_some: false,
            value: None,
            seed: None,
        }
    }

    pub fn with_seed<S>(self, seed: S) -> StringOrValue<T, Dst, S> {
        StringOrValue {
            string_dst: self.string_dst,
            is_some: self.is_some,
            value: self.value,
            seed: Some(seed),
        }
    }
}

impl<V, T, S> StringOrValue<V, ParsableDst<T>, S>
where
    T: FromStr,
{
    #[inline]
    pub const fn new_parsable() -> Self {
        Self {
            string_dst: ParsableDst(None),
            is_some: false,
            value: None,
            seed: None,
        }
    }

    #[inline]
    pub const fn new_parsable_with_seed(seed: S) -> Self {
        Self {
            string_dst: ParsableDst(None),
            is_some: false,
            value: None,
            seed: Some(seed),
        }
    }
}

impl<T, Dst, S> StringOrValue<T, Dst, S>
where
    T: AsRef<str>,
    Dst: AsRef<str>,
{
    #[inline]
    pub fn peek_string(&self) -> Option<&str> {
        if !self.is_some {
            return None;
        }

        match self.value.as_ref() {
            Some(val) => Some(val.as_ref()),
            None => Some(self.string_dst.as_ref()),
        }
    }
}

impl<T, Dst, S> StringOrValue<T, Dst, S>
where
    Dst: StringDst,
{
    pub fn new_with_seed(string_dst: Dst, seed: S) -> Self {
        Self {
            string_dst,
            is_some: false,
            value: None,
            seed: Some(seed),
        }
    }

    pub fn insert_seed(&mut self, seed: S) {
        self.seed = Some(seed);
    }

    pub fn is_some(&self) -> bool {
        self.is_some
    }

    pub fn reset(&mut self) -> Option<T> {
        self.is_some = false;
        self.string_dst.clear();
        self.value.take()
    }

    pub fn peek_result(&self) -> Option<StringOrValueResult<&T, &Dst>> {
        if self.is_some {
            match self.value.as_ref() {
                Some(value) => Some(StringOrValueResult::Value(value)),
                None => Some(StringOrValueResult::String(&self.string_dst)),
            }
        } else {
            None
        }
    }

    pub fn into_result(self) -> Option<StringOrValueResult<T, Dst>> {
        if self.is_some {
            match self.value {
                Some(value) => Some(StringOrValueResult::Value(value)),
                None => Some(StringOrValueResult::String(self.string_dst)),
            }
        } else {
            None
        }
    }

    pub fn take_result(&mut self) -> Option<StringOrValueResult<T, Dst>>
    where
        Dst: Default,
    {
        if self.is_some {
            self.is_some = false;

            match self.value.take() {
                Some(value) => Some(StringOrValueResult::Value(value)),
                None => Some(StringOrValueResult::String(std::mem::take(
                    &mut self.string_dst,
                ))),
            }
        } else {
            None
        }
    }
}

impl<T, Dst> Default for StringOrValue<T, Dst>
where
    Dst: Default,
{
    fn default() -> Self {
        Self::new(Dst::default())
    }
}

/// The result from deserializing a value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringOrValueResult<T, Dst> {
    String(Dst),
    Value(T),
}

impl<'de, T, Dst, S> de::DeserializeSeed<'de> for &mut StringOrValue<T, Dst, S>
where
    T: Deserialize<'de>,
    Dst: StringDst,
    S: de::DeserializeSeed<'de, Value = T> + de::Visitor<'de, Value = T>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

macro_rules! impl_visitor_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<E>(self, arg: $arg_ty) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let result = if let Some(seed) = self.seed.take() {
                    seed.$fn_name(arg)
                } else {
                    T::deserialize(arg.into_deserializer())
                };

                match result {
                    Ok(val) => {
                        self.value = Some(val);
                        self.is_some = true;
                        Ok(())
                    },
                    Err(err) => Err(err),
                }
            }
        )*
    };
}

impl<'de, T, Dst, V> de::Visitor<'de> for &mut StringOrValue<T, Dst, V>
where
    T: Deserialize<'de>,
    Dst: StringDst,
    V: de::DeserializeSeed<'de, Value = T> + de::Visitor<'de, Value = T>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string or value")
    }

    impl_visitor_fns! {
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_f32(f32),
        visit_f64(f64),
        visit_char(char),
        visit_bool(bool),
    }

    serde::serde_if_integer128! {
        impl_visitor_fns! {
            visit_i128(i128),
            visit_u128(u128),
        }
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let result = if let Some(seed) = self.seed.take() {
            seed.visit_unit()
        } else {
            T::deserialize(().into_deserializer())
        };

        match result {
            Ok(val) => {
                self.value = Some(val);
                self.is_some = true;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Some(seed) = self.seed.take() {
            self.value = Some(seed.visit_none()?);
            self.is_some = true;
        }

        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Some(seed) = self.seed.take() {
            let val = seed.deserialize(deserializer)?;
            self.value = Some(val);
            self.is_some = true;
            Ok(())
        } else {
            deserializer.deserialize_any(self)
        }
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        if let Some(seed) = self.seed.take() {
            self.value = Some(seed.visit_enum(data)?);
            self.is_some = true;
            Ok(())
        } else {
            Err(de::Error::custom(
                "StringOrValue cannot handle 'visit_enum' without a visitor",
            ))
        }
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.value = match self.seed.take() {
            Some(seed) => seed.visit_seq(seq).map(Some)?,
            _ => T::deserialize(de::value::SeqAccessDeserializer::new(seq)).map(Some)?,
        };

        self.is_some = true;
        Ok(())
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.value = match self.seed.take() {
            Some(seed) => seed.visit_map(map).map(Some)?,
            _ => T::deserialize(de::value::MapAccessDeserializer::new(map)).map(Some)?,
        };

        self.is_some = true;
        Ok(())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let result: Result<T, E> = if let Some(seed) = self.seed.take() {
            seed.visit_str(v)
        } else {
            T::deserialize(v.into_deserializer())
        };

        if let Ok(value) = result {
            self.value = Some(value);
        } else {
            self.string_dst.handle_str(v);
        }

        self.is_some = true;
        Ok(())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut take_str_des = TakeStringDeserializer { string: Some(v) };

        let opt = if let Some(seed) = self.seed.take() {
            seed.deserialize(&mut take_str_des).ok()
        } else {
            T::deserialize(&mut take_str_des).ok()
        };

        if let Some(value) = opt {
            self.value = Some(value);
            self.is_some = true;
        } else if let Some(s) = take_str_des.string.take() {
            self.string_dst.handle_string(s);
            self.is_some = true;
        }

        Ok(())
    }
}

struct TakeStringDeserializer {
    string: Option<String>,
}

impl<'de> de::Deserializer<'de> for &mut TakeStringDeserializer {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.string.as_ref() {
            Some(string) => visitor.visit_str(string.as_str()),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.string.take() {
            Some(string) => visitor.visit_string(string),
            None => visitor.visit_none(),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
