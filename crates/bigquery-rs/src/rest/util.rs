use std::marker::PhantomData;

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
