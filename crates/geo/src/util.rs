//! Utilities, including deserialization helpers, etc.

use std::fmt;

use serde::de;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexVisitor;

impl<'de> de::DeserializeSeed<'de> for IndexVisitor {
    type Value = usize;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

macro_rules! impl_primitive_visitors {
    ($($fn_name:ident($type:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<E>(self, arg: $type) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                match arg.try_into() {
                    Ok(converted) => Ok(converted),
                    Err(error) => Err(de::Error::custom(error)),
                }
            }
        )*
    };
}

impl<'de> de::Visitor<'de> for IndexVisitor {
    type Value = usize;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer index, encoded as either an integer or string")
    }

    impl_primitive_visitors! {
        visit_i8(i8),
        visit_u8(u8),
        visit_i16(i16),
        visit_u16(u16),
        visit_i32(i32),
        visit_u32(u32),
        visit_i64(i64),
        visit_u64(u64),
    }

    serde::serde_if_integer128! {
        impl_primitive_visitors! {
            visit_i128(i128),
            visit_u128(u128),
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let v = v.round();

        if v.is_sign_negative() {
            Err(de::Error::invalid_value(
                de::Unexpected::Float(v),
                &"index cannot be negative",
            ))
        } else if v >= usize::MAX as f64 {
            Err(de::Error::invalid_value(
                de::Unexpected::Float(v),
                &"index cannot be larger than `usize::MAX`",
            ))
        } else {
            Ok(v as usize)
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.trim().parse::<usize>() {
            Ok(parsed) => Ok(parsed),
            Err(error) => Err(de::Error::invalid_value(
                de::Unexpected::Str(v),
                &ExpectedDisplay(&error),
            )),
        }
    }
}

pub struct ExpectedDisplay<'a, T>(&'a T);

impl<T> de::Expected for ExpectedDisplay<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
