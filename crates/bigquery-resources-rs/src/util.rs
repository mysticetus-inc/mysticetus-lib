use serde::de;

// used for `#[serde(skip_serializing_if = "is_false")]` attrs
#[inline]
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}

#[inline]
fn int_try_into<T, O, E>(
    input: T,
    into_unexpected: impl FnOnce(T) -> serde::de::Unexpected<'static>,
) -> Result<O, E>
where
    T: TryInto<O> + Copy,
    T::Error: std::fmt::Display,
    E: serde::de::Error,
{
    input
        .try_into()
        .map_err(|err| E::invalid_value(into_unexpected(input), &err.to_string().as_str()))
}

/// helper trait for getting [`itoa::Integer`] compatible values
/// generically from NonZero variants.
///
/// TODO: remove if NonZero variants get [`itoa::Integer`] impls
pub trait IntoInteger: Copy + TryFrom<Self::Integer> {
    type Integer: itoa::Integer;

    fn into_integer(self) -> Self::Integer;
}

macro_rules! impl_into_integer_simple {
    ($($t:ty),* $(,)?) => {
        $(
            impl IntoInteger for $t {
                type Integer = $t;
                #[inline]
                fn into_integer(self) -> Self::Integer {
                    self
                }
            }
        )*
    };
}

impl_into_integer_simple!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

macro_rules! impl_into_integer_for_non_zeros {
    ($($nz:ident: $i:ty),* $(,)?) => {
        $(
            impl IntoInteger for std::num::$nz {
                type Integer = $i;
                #[inline]
                fn into_integer(self) -> Self::Integer {
                    self.get()
                }
            }
        )*
    };
}

impl_into_integer_for_non_zeros! {
    NonZeroU8: u8,
    NonZeroU16: u16,
    NonZeroU32: u32,
    NonZeroU64: u64,
    NonZeroU128: u128,
    NonZeroUsize: usize,
    NonZeroI8: i8,
    NonZeroI16: i16,
    NonZeroI32: i32,
    NonZeroI64: i64,
    NonZeroI128: i128,
    NonZeroIsize: isize,
}

macro_rules! define_int64_uint64_visitors {
    ($($name:ident : $unexpected_variant:ident => $output:ty),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $name;

            impl<'de> serde::de::DeserializeSeed<'de> for $name {
                type Value = $output;
                #[inline]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_any(self)
                }
            }

            impl<'de> serde::de::Visitor<'de> for $name {
                type Value = $output;

                #[inline]
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("an i64 or string formatted i64")
                }

                #[inline]
                fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    int_try_into(v, |v| de::Unexpected::$unexpected_variant(v as _))
                }

                #[inline]
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    v.parse::<$output>().map_err(|err| {
                        E::invalid_value(de::Unexpected::Str(v), &err.to_string().as_str())
                    })
                }

                #[inline]
                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    let s = std::str::from_utf8(v).map_err(|err| {
                        E::invalid_value(de::Unexpected::Bytes(v), &err.to_string().as_str())
                    })?;
                    self.visit_str(s)
                }
            }
        )*
    };
}

define_int64_uint64_visitors! {
    Int64ValueVisitor:Signed => i64,
    Uint64ValueVisitor:Unsigned => u64,
}

pub(crate) mod int64 {
    #[allow(dead_code)]
    #[inline]
    pub fn serialize<I, S>(int: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        I: super::IntoInteger + Copy,
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(int.into_integer()))
    }

    #[allow(dead_code)]
    #[inline]
    pub fn deserialize<'de, D, I>(deserializer: D) -> Result<I, D::Error>
    where
        I: TryFrom<i64>,
        I::Error: std::fmt::Display,
        D: serde::Deserializer<'de>,
    {
        let int = deserializer.deserialize_any(super::Int64ValueVisitor)?;

        I::try_from(int).map_err(|err| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Signed(int),
                &err.to_string().as_str(),
            )
        })
    }

    pub mod optional {
        #[inline]
        pub fn serialize<I, S>(int: &Option<I>, serializer: S) -> Result<S::Ok, S::Error>
        where
            I: super::super::IntoInteger + Copy,
            S: serde::Serializer,
        {
            match *int {
                Some(int) => {
                    serializer.serialize_some(itoa::Buffer::new().format(int.into_integer()))
                }
                None => serializer.serialize_none(),
            }
        }

        #[inline]
        pub fn deserialize<'de, D, I>(deserializer: D) -> Result<Option<I>, D::Error>
        where
            I: super::super::IntoInteger,
            I::Integer: TryFrom<i64>,
            <I::Integer as TryFrom<i64>>::Error: std::fmt::Display,
            I::Error: std::fmt::Display,
            D: serde::Deserializer<'de>,
        {
            fn make_map_err<E: std::fmt::Display, D: serde::de::Error>(
                int: i64,
            ) -> impl FnOnce(E) -> D {
                move |err| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(int),
                        &err.to_string().as_str(),
                    )
                }
            }

            let maybe_int = deserializer.deserialize_any(
                serde_helpers::optional_visitor::OptionalVisitor::from(
                    super::super::Int64ValueVisitor,
                ),
            )?;

            let Some(int) = maybe_int else {
                return Ok(None);
            };

            let half_mapped =
                <I::Integer as TryFrom<i64>>::try_from(int).map_err(make_map_err(int))?;

            I::try_from(half_mapped)
                .map(Some)
                .map_err(make_map_err(int))
        }
    }
}

pub(crate) mod uint64 {
    #[allow(dead_code)]
    #[inline]
    pub fn serialize<I, S>(int: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        I: super::IntoInteger + Copy,
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(int.into_integer()))
    }

    #[allow(dead_code)]
    #[inline]
    pub fn deserialize<'de, D, I>(deserializer: D) -> Result<I, D::Error>
    where
        I: TryFrom<u64>,
        I::Error: std::fmt::Display,
        D: serde::Deserializer<'de>,
    {
        let uint = deserializer.deserialize_any(super::Uint64ValueVisitor)?;

        I::try_from(uint).map_err(|err| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(uint),
                &err.to_string().as_str(),
            )
        })
    }

    pub mod optional {
        #[inline]
        pub fn serialize<I, S>(int: &Option<I>, serializer: S) -> Result<S::Ok, S::Error>
        where
            I: super::super::IntoInteger + Copy,
            S: serde::Serializer,
        {
            match *int {
                Some(int) => {
                    serializer.serialize_some(itoa::Buffer::new().format(int.into_integer()))
                }
                None => serializer.serialize_none(),
            }
        }

        #[inline]
        pub fn deserialize<'de, D, I>(deserializer: D) -> Result<Option<I>, D::Error>
        where
            I: TryFrom<u64>,
            I::Error: std::fmt::Display,
            D: serde::Deserializer<'de>,
        {
            let maybe_uint = deserializer.deserialize_any(
                serde_helpers::optional_visitor::OptionalVisitor::from(
                    super::super::Uint64ValueVisitor,
                ),
            )?;

            let Some(uint) = maybe_uint else {
                return Ok(None);
            };

            I::try_from(uint).map(Some).map_err(|err| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Unsigned(uint),
                    &err.to_string().as_str(),
                )
            })
        }
    }
}

pub(crate) mod duration_ms {
    use serde::Serialize;
    use timestamp::Duration;

    #[allow(dead_code)]
    #[inline]
    pub fn serialize<S>(timeout: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializeAsInt64Millis(*timeout).serialize(serializer)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(super::Int64ValueVisitor)
            .map(Duration::from_millis_i64_saturating)
    }

    struct SerializeAsInt64Millis(Duration);

    impl serde::Serialize for SerializeAsInt64Millis {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let millis = self.0.millis();
            itoa::Buffer::new().format(millis).serialize(serializer)
        }
    }

    pub mod optional {
        pub fn serialize<S>(
            timeout: &Option<timestamp::Duration>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match *timeout {
                Some(timeout) => serializer.serialize_some(&super::SerializeAsInt64Millis(timeout)),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<timestamp::Duration>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let optional = deserializer.deserialize_option(
                serde_helpers::optional_visitor::OptionalVisitor::from(
                    super::super::Int64ValueVisitor,
                ),
            )?;

            Ok(optional.map(timestamp::Duration::from_millis_i64_saturating))
        }
    }
}

pub mod timestamp_ms {
    #[allow(dead_code)]
    #[inline]
    pub fn serialize<S>(ts: &timestamp::Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(ts.as_millis()))
    }

    #[allow(dead_code)]
    #[inline]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<timestamp::Timestamp, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(super::Int64ValueVisitor)
            .map(timestamp::Timestamp::from_millis_saturating)
    }

    pub mod optional {
        #[inline]
        pub fn serialize<S>(
            ts: &Option<timestamp::Timestamp>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match ts {
                Some(ts) => serializer.serialize_some(itoa::Buffer::new().format(ts.as_millis())),
                None => serializer.serialize_none(),
            }
        }

        #[inline]
        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<Option<timestamp::Timestamp>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer
                .deserialize_any(serde_helpers::optional_visitor::OptionalVisitor::from(
                    super::super::Int64ValueVisitor,
                ))
                .map(|opt| opt.map(timestamp::Timestamp::from_millis_saturating))
        }
    }
}
