use std::marker::PhantomData;

use serde::de;

#[repr(u32)]
pub enum GeomType {
    Point = 1,
    LineString = 2,
    Polygon = 3,
    MultiPoint = 4,
    MultiLineString = 5,
    MultiPolygon = 6,
}

/// capacity needed for the LE/BE tag + geometry type tag.
const TAG_SIZE: usize = 1 + 4;
const COORD_SIZE: usize = std::mem::size_of::<f64>();

/// Type that performs the needed serialization to insert a Geography point into BQ.
///
/// Use the #[serde(serialize_with = "to_bigquery_wkb_point")] function to serialize
/// anything that's both [`Copy`] + [`Into<WkbPoint>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WkbPoint([u8; Self::ARRAY_LEN]);

/// Helper function that performs serialization for points, compatible with the serde
/// `serialize_with` field attribute.
pub fn to_bigquery_wkb_point<P, S>(point: &P, serializer: S) -> Result<S::Ok, S::Error>
where
    P: Copy + Into<WkbPoint>,
    S: serde::Serializer,
{
    let converted: WkbPoint = (*point).into();
    serde::Serialize::serialize(&converted, serializer)
}

impl serde::Serialize for WkbPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = [0; 2 * Self::ARRAY_LEN];
        if let Err(err) = hex::encode_to_slice(&self.0, &mut buf) {
            return Err(serde::ser::Error::custom(err));
        }

        match std::str::from_utf8(&buf) {
            Ok(s) => serializer.collect_str(s),
            Err(e) => Err(serde::ser::Error::custom(e)),
        }
    }
}

impl WkbPoint {
    const ARRAY_LEN: usize = TAG_SIZE + 2 * COORD_SIZE;

    const BASE_ARRAY: [u8; Self::ARRAY_LEN] = [
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const LON_START: usize = TAG_SIZE;
    const LAT_START: usize = TAG_SIZE + COORD_SIZE;

    pub fn builder() -> WkbPointBuilder<()> {
        WkbPointBuilder {
            dst: Self::BASE_ARRAY,
            _marker: PhantomData,
        }
    }

    pub fn from_point(lon: f64, lat: f64) -> Self {
        Self::builder().lon(lon).lat(lat)
    }
}

pub struct WkbPointBuilder<Lon> {
    dst: [u8; WkbPoint::ARRAY_LEN],
    _marker: PhantomData<Lon>,
}

impl WkbPointBuilder<()> {
    pub fn lon(mut self, lon: f64) -> WkbPointBuilder<f64> {
        self.dst[WkbPoint::LON_START..WkbPoint::LAT_START].copy_from_slice(&lon.to_be_bytes());

        WkbPointBuilder {
            dst: self.dst,
            _marker: PhantomData,
        }
    }
}

// used for `#[serde(skip_serializing_if = "is_false")]` attrs
#[inline]
pub fn is_false(b: &bool) -> bool {
    !*b
}

impl WkbPointBuilder<f64> {
    pub fn lat(mut self, lat: f64) -> WkbPoint {
        self.dst[WkbPoint::LAT_START..].copy_from_slice(&lat.to_be_bytes());
        WkbPoint(self.dst)
    }
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

macro_rules! define_int64_uint64_visitors {
    ($($name:ident : $unexpected_variant:ident => $output:ty),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $name;

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
    #[inline]
    pub fn serialize<I, S>(int: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        I: itoa::Integer + Copy,
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(*int))
    }

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
            I: itoa::Integer + Copy,
            S: serde::Serializer,
        {
            match *int {
                Some(int) => serializer.serialize_some(itoa::Buffer::new().format(int)),
                None => serializer.serialize_none(),
            }
        }

        #[inline]
        pub fn deserialize<'de, D, I>(deserializer: D) -> Result<Option<I>, D::Error>
        where
            I: TryFrom<i64>,
            I::Error: std::fmt::Display,
            D: serde::Deserializer<'de>,
        {
            let maybe_int = deserializer.deserialize_any(
                serde_helpers::optional_visitor::OptionalVisitor::from(
                    super::super::Int64ValueVisitor,
                ),
            )?;

            let Some(int) = maybe_int else {
                return Ok(None);
            };

            I::try_from(int).map(Some).map_err(|err| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Signed(int),
                    &err.to_string().as_str(),
                )
            })
        }
    }
}

pub(crate) mod uint64 {
    #[inline]
    pub fn serialize<I, S>(int: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        I: itoa::Integer + Copy,
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(*int))
    }

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
            I: itoa::Integer + Copy,
            S: serde::Serializer,
        {
            match *int {
                Some(int) => serializer.serialize_some(itoa::Buffer::new().format(int)),
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

    #[inline]
    pub fn serialize<S>(timeout: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializeAsInt64Millis(*timeout).serialize(serializer)
    }

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
    #[inline]
    pub fn serialize<S>(ts: &timestamp::Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(ts.as_millis()))
    }

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
