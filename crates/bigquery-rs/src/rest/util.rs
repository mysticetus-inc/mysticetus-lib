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

impl WkbPointBuilder<f64> {
    pub fn lat(mut self, lat: f64) -> WkbPoint {
        self.dst[WkbPoint::LAT_START..].copy_from_slice(&lat.to_be_bytes());
        WkbPoint(self.dst)
    }
}

pub struct Int64ValueVisitor;

impl<'de> serde::de::Visitor<'de> for Int64ValueVisitor {
    type Value = i64;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an i64 or string formatted i64")
    }

    #[inline]
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    #[inline]
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.try_into() {
            Ok(i) => self.visit_i64(i),
            Err(err) => Err(E::invalid_value(
                de::Unexpected::Unsigned(v),
                &err.to_string().as_str(),
            )),
        }
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<i64>()
            .map_err(|err| E::invalid_value(de::Unexpected::Str(v), &err.to_string().as_str()))
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = std::str::from_utf8(v)
            .map_err(|err| E::invalid_value(de::Unexpected::Bytes(v), &err.to_string().as_str()))?;
        self.visit_str(s)
    }
}

pub struct OptionalInt64ValueVisitor;

impl<'de> serde::de::Visitor<'de> for OptionalInt64ValueVisitor {
    type Value = Option<i64>;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an optional i64 or string formatted i64")
    }

    #[inline]
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(v))
    }

    #[inline]
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.try_into() {
            Ok(i) => self.visit_i64(i),
            Err(err) => Err(E::invalid_value(
                de::Unexpected::Unsigned(v),
                &err.to_string().as_str(),
            )),
        }
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<i64>()
            .map(Some)
            .map_err(|err| E::invalid_value(de::Unexpected::Str(v), &err.to_string().as_str()))
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = std::str::from_utf8(v)
            .map_err(|err| E::invalid_value(de::Unexpected::Bytes(v), &err.to_string().as_str()))?;
        self.visit_str(s)
    }

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
        deserializer.deserialize_any(self)
    }
}

pub(crate) mod timeout_ms {
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
            let optional =
                deserializer.deserialize_option(super::super::OptionalInt64ValueVisitor)?;

            Ok(optional.map(timestamp::Duration::from_millis_i64_saturating))
        }
    }
}
