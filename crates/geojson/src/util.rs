macro_rules! impl_str_marker_type {
    ($v:vis $t:ident: $s:literal) => {
        $v struct $t;

        impl<'de> ::serde::Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'vde> ::serde::de::Visitor<'vde> for Visitor {
                    type Value = $t;

                    fn expecting(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        f.write_str(concat!(
                            "the string '",
                            $s,
                            "'"
                        ))
                    }

                    fn visit_str<E>(self, st: &str) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error
                    {
                        if st == $s {
                            Ok($t)
                        } else {
                            Err(::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(st), &self))
                        }
                    }

                    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error
                    {
                        let s = ::std::str::from_utf8(b).map_err(::serde::de::Error::custom)?;
                        self.visit_str(s)
                    }
                }


                deserializer.deserialize_str(Visitor)
            }
        }
    };
}

macro_rules! impl_field_name_from_str {
    (
        $t:ty {
            $(
                $variant:ident: $first_byte:pat => $rest_str:literal
            ),*
            $(,)?
        }
    ) => {
        impl ::std::str::FromStr for $t {
            type Err = ::std::convert::Infallible;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                fn finish_check(mut bytes: ::std::str::Bytes<'_>, expected: &[u8], if_eq: $t) -> $t {
                    if bytes.by_ref().zip(expected).all(|(a, b)| a.eq_ignore_ascii_case(b)) && bytes.next().is_none() {
                        if_eq
                    } else {
                        <$t>::Unknown
                    }
                }


                let mut bytes = s.bytes();

                match bytes.next() {
                    $(
                        Some($first_byte) => Ok(finish_check(bytes, $rest_str, Self::$variant)),
                    )*
                    _ => Ok(Self::Unknown),
                }
            }
        }
    };
}

pub(crate) use {impl_field_name_from_str, impl_str_marker_type};

pub(crate) fn drain_map_access<'de, M>(mut map_access: M) -> Result<(), M::Error>
where
    M: serde::de::MapAccess<'de>,
{
    use serde::de::IgnoredAny;

    while map_access.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}

    Ok(())
}
