use std::fmt;

pub use geo;
pub use geo::Point;
pub use geo::geom::{Line as LineString, Polygon};
pub use geo::util::IndexVisitor;

pub mod any;
pub use any::AnyCoordinate;
use serde::{Deserialize, Serialize, de, ser};
use serde_helpers::from_str_visitor::FromStrVisitor;

/// Trait implemented by any valid GeoJson coordinate.
pub trait Coordinate {
    /// Returns the [`GeometryType`] for [`Self`].
    fn geometry_type(&self) -> GeometryType;

    /// Wraps [`Self`] in [`Geometry`] for serialization.
    fn as_geometry(&self) -> Geometry<&'_ Self> {
        Geometry {
            geometry_type: self.geometry_type(),
            coordinates: self,
        }
    }
}

impl<T> Coordinate for &T
where
    T: Coordinate,
{
    fn geometry_type(&self) -> GeometryType {
        T::geometry_type(self)
    }
}

impl<T> Coordinate for Box<T>
where
    T: Coordinate,
{
    fn geometry_type(&self) -> GeometryType {
        T::geometry_type(self)
    }
}

/// Helper trait to handle serializing nested coordinates as maps instead of nested arrays.
pub trait MapCoordinate: Coordinate + Serialize {
    /// An implementor defined helper for serializing nested coordinates as a map instead of an
    /// array.
    type MapCoordinate<'a>: Serialize
    where
        Self: 'a;

    /// Returns the map-serializable version of [`Self`].
    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_>;

    /// Wraps [`Self`] in both the [`Self::MapCoordinate`] + [`Geometry`] for serializing
    /// [`Self`] as a map.
    fn as_map_geometry(&self) -> Geometry<Self::MapCoordinate<'_>> {
        Geometry {
            geometry_type: self.geometry_type(),
            coordinates: self.as_map_coordinate(),
        }
    }
}

impl<T> MapCoordinate for &T
where
    T: MapCoordinate,
{
    type MapCoordinate<'a>
        = T::MapCoordinate<'a>
    where
        Self: 'a;

    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_> {
        T::as_map_coordinate(self)
    }
}

impl<T> MapCoordinate for Box<T>
where
    T: MapCoordinate,
{
    type MapCoordinate<'a>
        = T::MapCoordinate<'a>
    where
        Self: 'a;

    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_> {
        T::as_map_coordinate(self)
    }
}

/// The supported Geometry types.
///
/// In the future, MultiPoint, MultiLineString, etc, will be added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum GeometryType {
    Point,
    LineString,
    Polygon,
}

impl serde::de::Expected for GeometryType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{self} geometry")
    }
}

impl fmt::Display for GeometryType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl GeometryType {
    /// Returns a [`&'static str`] with the name of the geometry type. The string returned here
    /// is valid for the 'type' key in GeoJson Geometry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::LineString => "LineString",
            Self::Polygon => "Polygon",
        }
    }
}

impl Serialize for GeometryType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Serialization helper/wrapper for any [`Coordinate`].
///
/// This takes a normal coordinate, and makes it to serialize as valid GeoJson geometry rather
/// than just as the coordinate itself.
#[derive(Clone)]
pub struct Geometry<T> {
    geometry_type: GeometryType,
    coordinates: T,
}

impl<T> Geometry<T> {
    pub(crate) fn into_coordinates(self) -> T {
        self.coordinates
    }
}

impl<T> fmt::Debug for Geometry<T>
where
    T: Coordinate + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Geometry")
            .field("type", &self.geometry_type.as_str())
            .field("coordinates", &self.coordinates)
            .finish()
    }
}

impl<T> Serialize for Geometry<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map_ser = serializer.serialize_map(Some(2))?;

        map_ser.serialize_entry("type", self.geometry_type.as_str())?;
        map_ser.serialize_entry("coordinates", &self.coordinates)?;

        map_ser.end()
    }
}

impl<'de, T> Deserialize<'de> for Geometry<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor {
            _marker: std::marker::PhantomData,
        })
    }
}

struct GeometryVisitor<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<'de, T> de::Visitor<'de> for GeometryVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Geometry<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "valid GeoJson geometry")
    }

    fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        enum GeometryFields {
            Type,
            Coordinates,
            Unknown,
        }

        crate::util::impl_field_name_from_str! {
            GeometryFields {
                Type: b'T' | b't' => b"ype",
                Coordinates: b'C' | b'c' => b"oordinates",
            }
        }

        let mut geometry_type = None;
        let mut coordinates = None;

        while let Some(field) = map_access.next_key_seed(FromStrVisitor::<GeometryFields>::new())? {
            let should_skip_remaining = match field {
                GeometryFields::Type => {
                    if geometry_type.is_some() {
                        return Err(de::Error::duplicate_field("type"));
                    }

                    geometry_type = Some(map_access.next_value::<GeometryType>()?);
                    coordinates.is_some()
                }
                GeometryFields::Coordinates => {
                    if coordinates.is_some() {
                        return Err(de::Error::duplicate_field("coordinates"));
                    }
                    coordinates = Some(map_access.next_value()?);
                    geometry_type.is_some()
                }
                _ => {
                    // we still need to drain the value, even if its not something we care about
                    let _: de::IgnoredAny = map_access.next_value()?;
                    continue;
                }
            };

            if should_skip_remaining {
                // we cant technically break early, since it'll leave the cursor in the
                // deserializer in a weird spot when it tries to start deserializing again,
                // causing errors.
                //
                // In order to bail early, we need to drain all other keys/values in the map,
                // then bail. Using de::IgnoredAny this draining is very efficient.

                crate::util::drain_map_access(&mut map_access)?;
                break;
            }
        }

        let geometry_type = geometry_type.ok_or_else(|| de::Error::missing_field("type"))?;
        let coordinates = coordinates.ok_or_else(|| de::Error::missing_field("coordinates"))?;

        Ok(Geometry {
            geometry_type,
            coordinates,
        })
    }
}

impl Coordinate for Point {
    fn geometry_type(&self) -> GeometryType {
        GeometryType::Point
    }
}

impl MapCoordinate for Point {
    type MapCoordinate<'a> = &'a Self;

    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_> {
        self
    }
}

impl Coordinate for LineString {
    fn geometry_type(&self) -> GeometryType {
        GeometryType::LineString
    }
}

impl MapCoordinate for LineString {
    type MapCoordinate<'a> = NestedMapCoordinate<'a, Point>;

    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_> {
        NestedMapCoordinate(self.as_slice())
    }
}

impl Coordinate for Polygon {
    fn geometry_type(&self) -> GeometryType {
        GeometryType::Polygon
    }
}

impl MapCoordinate for Polygon {
    type MapCoordinate<'a> = NestedMapCoordinate<'a, LineString>;

    fn as_map_coordinate(&self) -> Self::MapCoordinate<'_> {
        NestedMapCoordinate(self.as_slice())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NestedMapCoordinate<'a, T>(&'a [T]);

impl<T> Serialize for NestedMapCoordinate<'_, T>
where
    T: MapCoordinate,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut buf = itoa::Buffer::new();

        let mut map_ser = serializer.serialize_map(Some(self.0.len()))?;

        for (idx, nested) in self.0.iter().enumerate() {
            let idx_str = buf.format(idx);
            map_ser.serialize_entry(idx_str, &nested.as_map_coordinate())?;
        }

        map_ser.end()
    }
}
