pub mod with {
    use ::serde::de::{self, Expected, Unexpected};
    use ::std::fmt;

    macro_rules! int_try_from_fns {
        (
            $($fn_name:ident($arg_ty:ty)),*
            $(,)?
        ) => {
            $(
                fn $fn_name<E>(self, value: $arg_ty) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match TryFrom::try_from(value) {
                        Ok(converted) => Ok(converted),
                        Err(err) => Err(serde::de::Error::invalid_value(super::UnexpectedHelper::from(value).0, &super::ExpectingWrapper(&err))),
                    }
                }
            )*
        };
    }

    macro_rules! int_to_float_cast_fns {
        (
            $($fn_name:ident($arg_ty:ty)),*
            $(,)?
        ) => {
            $(
                fn $fn_name<E>(self, value: $arg_ty) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(value as f64)
                }
            )*
        };
    }

    struct ExpectingWrapper<'a, T>(&'a T);

    impl<T: fmt::Display> Expected for ExpectingWrapper<'_, T> {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(&self.0, formatter)
        }
    }

    // helper type to provide From impls for the types it can contain
    struct UnexpectedHelper<'a>(Unexpected<'a>);

    macro_rules! impl_unexpected_helper_from {
        ($($from_ty:ty: $arg:ident => $blk:expr),* $(,)?) => {
            $(
                impl<'a> From<$from_ty> for UnexpectedHelper<'a> {
                    fn from($arg: $from_ty) -> Self {
                        Self($blk)
                    }
                }
            )*
        };
    }

    impl_unexpected_helper_from! {
        i8: int => Unexpected::Signed(int as i64),
        i16: int => Unexpected::Signed(int as i64),
        i32: int => Unexpected::Signed(int as i64),
        i64: int => Unexpected::Signed(int as i64),
        i128: int => Unexpected::Signed(int as i64),
        u8: uint => Unexpected::Unsigned(uint as u64),
        u16: uint => Unexpected::Unsigned(uint as u64),
        u32: uint => Unexpected::Unsigned(uint as u64),
        u64: uint => Unexpected::Unsigned(uint as u64),
        u128: uint => Unexpected::Unsigned(uint as u64),
        f32: double => Unexpected::Float(double as f64),
        f64: double => Unexpected::Float(double as f64),
    }

    struct OptionalVisitor<V>(V);

    impl<'de, V> de::Visitor<'de> for OptionalVisitor<V>
    where
        V: de::Visitor<'de>,
    {
        type Value = Option<V::Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            self.0.expecting(formatter)?;
            formatter.write_str(" (optional)")
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

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self.0).map(Some)
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self.0).map(Some)
        }
    }

    macro_rules! float_noop {
        ($num_ty:ty; $visitor:expr, $value:expr) => {{ Ok($value as $num_ty) }};
    }

    macro_rules! float_to_int {
        ($num_ty:ty; $visitor:expr, $value:expr) => {{
            const MIN: f64 = <$num_ty>::MIN as f64;
            const MAX: f64 = <$num_ty>::MAX as f64;

            let rounded = $value.round();
            if (MIN..=MAX).contains(&rounded) {
                Ok(rounded as $num_ty)
            } else {
                Err(de::Error::invalid_value(
                    super::UnexpectedHelper::from($value).0,
                    &$visitor,
                ))
            }
        }};
    }

    macro_rules! impl_mod {
        ($($num_ty:ty => { $mod_name:ident, $signed_unsigned_str:literal, $int_macro:ident, $float_macro:ident }),* $(,)?) => {
            $(
                pub mod $mod_name {
                    use serde::{de, Deserializer, Serializer};
                    use std::fmt;

                    #[allow(dead_code)]
                    pub fn serialize<S>(int: &$num_ty, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer,
                    {
                        serializer.collect_str(int)
                    }

                    #[allow(dead_code)]
                    pub fn deserialize<'de, D>(deserializer: D) -> Result<$num_ty, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        deserializer.deserialize_str(Visitor)
                    }

                    struct Visitor;

                    impl<'de> de::Visitor<'de> for Visitor {
                        type Value = $num_ty;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(
                                concat!("a ", $signed_unsigned_str, ", either as a number or string")
                            )
                        }

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            match v.trim().parse::<$num_ty>() {
                                Ok(v) => Ok(v),
                                Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
                            }
                        }

                        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
                        where
                            E: de::Error
                        {
                            $float_macro!($num_ty; self, v)
                        }

                        $int_macro! {
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
                        }
                    }

                    pub mod option {
                        use super::super::OptionalVisitor;
                        use super::*;

                        #[allow(dead_code)]
                        pub fn serialize<S>(int: &Option<$num_ty>, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: Serializer,
                        {
                            match int {
                                Some(int) => serializer.collect_str(int),
                                None => serializer.serialize_none(),
                            }
                        }

                        #[allow(dead_code)]
                        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<$num_ty>, D::Error>
                        where
                            D: Deserializer<'de>,
                        {
                            deserializer.deserialize_option(OptionalVisitor(Visitor))
                        }
                    }
                }

            )*
        };
    }

    impl_mod! {
        f64 => { double, "double", int_to_float_cast_fns, float_noop },
        i64 => { int64, "signed integer (up to 64 bit)", int_try_from_fns, float_to_int },
        u64 => { uint64, "unsigned integer (up to 64 bit)", int_try_from_fns, float_to_int },
        i32 => { int32, "signed integer", int_try_from_fns, float_to_int },
        u32 => { uint32, "unsigned integer", int_try_from_fns, float_to_int },
    }
}
