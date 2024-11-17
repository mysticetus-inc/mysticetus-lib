//! [`Feature`] definition + impls
use std::fmt;

use path_aware_serde::{Deserializer as PathDeserializer, Error};
use serde::ser::{self, SerializeMap};
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_helpers::from_str_visitor::FromStrVisitor;

use crate::builder::FeatureBuilder;
use crate::geometry::any::{AnyCoordinate, GenericCoordinate};
use crate::geometry::{Coordinate, Geometry, GeometryType, MapCoordinate};
use crate::macros::de_err;
use crate::properties::Properties;

/// A single GeoJson Feature
///
/// The 'geometry' field is flattened outside of serialization/deserialization, leading to
/// `coordinates` being in the root of the Feature.
///
/// Both the coordinates and properties are generic, though most methods requiring that they
/// implement [`Coordinate`] and [`Properties`], respectively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature<C, P> {
    coordinates: C,
    properties: P,
}

/// A wrapper around a `&[`Feature`]` that serializes the coordinates as a map, instead of nested
/// arrays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapFeatureRef<'a, C, P> {
    inner: &'a Feature<C, P>,
}

/// A wrapper similar to [`MapFeatureRef`], except this takes an owned feature.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MapFeature<C, P> {
    #[serde(
        flatten,
        bound = "C: MapCoordinate + Deserialize<'de>, P: Deserialize<'de>"
    )]
    inner: Feature<C, P>,
}

impl<C, P> MapFeature<C, P> {
    /// Returns the inner feature.
    pub fn into_inner(self) -> Feature<C, P> {
        self.inner
    }
}

impl Feature<(), ()> {
    /// Returns an empty [`FeatureBuilder`].
    ///
    /// Shortcut to [`FeatureBuilder::empty`] to avoid requiring the extra 'use' statement.
    pub fn builder() -> FeatureBuilder<(), ()> {
        FeatureBuilder::empty()
    }
}

impl<C, P> Feature<C, P>
where
    P: Properties,
{
    pub fn build_with_args(args: P::RequiredArgs) -> FeatureBuilder<Option<C>, P> {
        FeatureBuilder::new_with_args(args).with_coordinates(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertError<C, P> {
    Coords(C),
    Props(P),
}

impl<E> ConvertError<E, E> {
    pub fn flatten(self) -> E {
        match self {
            Self::Coords(c) => c,
            Self::Props(p) => p,
        }
    }
}

impl<C, P> ConvertError<C, P> {
    pub fn into_error<E>(self) -> E
    where
        E: From<C> + From<P>,
    {
        match self {
            Self::Coords(c) => E::from(c),
            Self::Props(p) => E::from(p),
        }
    }
}

impl<C, P> Feature<C, P> {
    /// Takes a modifier function to wrap a reference to the properties. Used to help customize
    /// serialization/deserialization.
    pub fn wrap_props<F, P2>(&self, modifier_fn: F) -> Feature<&C, P2>
    where
        F: FnOnce(&P) -> P2,
    {
        Feature {
            coordinates: &self.coordinates,
            properties: modifier_fn(&self.properties),
        }
    }

    /// Uses a [`TryFrom`] impl to convert the coordinates to a new type.
    pub fn try_convert_coords<C2>(self) -> Result<Feature<C2, P>, C2::Error>
    where
        C2: TryFrom<C>,
    {
        let coordinates = self.coordinates.try_into()?;

        Ok(Feature {
            coordinates,
            properties: self.properties,
        })
    }

    /// Uses a [`TryFrom`] impl to convert the properties to a new type.
    pub fn try_convert_props<P2>(self) -> Result<Feature<C, P2>, P2::Error>
    where
        P2: TryFrom<P>,
    {
        let properties = self.properties.try_into()?;

        Ok(Feature {
            coordinates: self.coordinates,
            properties,
        })
    }

    /// Uses a [`TryFrom`] impl to convert both the coordinates and properties to a new type.
    /// If one impl throws an error, the corresponding failed field is represented in the error
    /// enum.
    pub fn try_convert<C2, P2>(self) -> Result<Feature<C2, P2>, ConvertError<C2::Error, P2::Error>>
    where
        C2: TryFrom<C>,
        P2: TryFrom<P>,
    {
        let coordinates = self.coordinates.try_into().map_err(ConvertError::Coords)?;
        let properties = self.properties.try_into().map_err(ConvertError::Props)?;

        Ok(Feature {
            coordinates,
            properties,
        })
    }

    /// Uses a [`From`] impl to convert the coordinates into a new type.
    pub fn convert_coords<C2>(self) -> Feature<C2, P>
    where
        C2: From<C>,
    {
        Feature {
            coordinates: self.coordinates.into(),
            properties: self.properties,
        }
    }

    /// Uses a [`From`] impl to convert the properties into a new type.
    pub fn convert_props<P2>(self) -> Feature<C, P2>
    where
        P2: From<P>,
    {
        Feature {
            coordinates: self.coordinates,
            properties: self.properties.into(),
        }
    }

    /// Uses a [`From`] impl to convert both the coordinates and properties into a new type.
    pub fn convert<C2, P2>(self) -> Feature<C2, P2>
    where
        C2: From<C>,
        P2: From<P>,
    {
        Feature {
            coordinates: self.coordinates.into(),
            properties: self.properties.into(),
        }
    }

    /// Takes a modifier function to wrap a mutable reference to the properties. Used to help
    /// customize serialization/deserialization.
    pub fn wrap_props_mut<F, P2>(&mut self, modifier_fn: F) -> Feature<&mut C, P2>
    where
        F: FnOnce(&mut P) -> P2,
    {
        Feature {
            coordinates: &mut self.coordinates,
            properties: modifier_fn(&mut self.properties),
        }
    }

    /// Takes a function that maps the coordinates, returning a new feature with the original
    /// properties, and resulting coordinates.
    pub fn map_coordinates<F, C2>(self, map_fn: F) -> Feature<C2, P>
    where
        F: FnOnce(C) -> C2,
    {
        Feature {
            coordinates: map_fn(self.coordinates),
            properties: self.properties,
        }
    }

    /// Takes a function that tries to maps the coordinates, returning a new feature with the
    /// original properties, and resulting coordinates, if 'map_fn' returns [`Ok`].
    pub fn try_map_coordinates<F, C2, E>(self, map_fn: F) -> Result<Feature<C2, P>, E>
    where
        F: FnOnce(C) -> Result<C2, E>,
    {
        let coordinates = map_fn(self.coordinates)?;

        Ok(Feature {
            coordinates,
            properties: self.properties,
        })
    }

    /// Takes a function that filters/maps the coordinates, returning a new feature with the
    /// original properties and resulting coordinates if the function returns [`Some`]. If the
    /// map function returns [`None`], the entire feature is filtered, returning [`None`].
    pub fn filter_map_coordinates<F, C2>(self, map_fn: F) -> Option<Feature<C2, P>>
    where
        F: FnOnce(C) -> Option<C2>,
    {
        let coordinates = map_fn(self.coordinates)?;

        Some(Feature {
            coordinates,
            properties: self.properties,
        })
    }

    /// Takes a function that maps the properties, returning a new feature with the original
    /// coordinates, and resulting properties.
    pub fn map_properties<F, P2>(self, map_fn: F) -> Feature<C, P2>
    where
        F: FnOnce(P) -> P2,
    {
        Feature {
            coordinates: self.coordinates,
            properties: map_fn(self.properties),
        }
    }

    /// Takes a function that filters/maps the properties, returning a new feature with the
    /// original coordinates and resulting properties if the function returns [`Some`]. If the
    /// map function returns [`None`], the entire feature is filtered, returning [`None`].
    pub fn filter_map_properties<F, P2>(self, map_fn: F) -> Option<Feature<C, P2>>
    where
        F: FnOnce(P) -> Option<P2>,
    {
        let properties = map_fn(self.properties)?;

        Some(Feature {
            coordinates: self.coordinates,
            properties,
        })
    }

    /// Takes a function that tries to map the properties to a new type. If the map function
    /// returns an [`Err`], this returns that error.
    pub fn try_map_properties<F, E, P2>(self, map_fn: F) -> Result<Feature<C, P2>, E>
    where
        F: FnOnce(P) -> Result<P2, E>,
    {
        let properties = map_fn(self.properties)?;
        Ok(Feature {
            coordinates: self.coordinates,
            properties,
        })
    }

    /// Returns a tuple with the raw coordinates and properties.
    pub fn into_inner(self) -> (C, P) {
        (self.coordinates, self.properties)
    }

    /// Turns this feature into a new feature, with the coordinate/properties wrapped in [`Some`].
    /// This opens up the [`take_coordinates`] + [`take_properties`] functions.
    ///
    /// [`take_coordinates`]: [`Feature::take_coordinates`]
    /// [`take_properties`]: [`Feature::take_properties`]
    pub fn into_optional(self) -> Feature<Option<C>, Option<P>> {
        Feature {
            coordinates: Some(self.coordinates),
            properties: Some(self.properties),
        }
    }

    /// Assembles the [`Feature`] from the passed in coordinates + properties
    pub fn from_coords_and_properties(coordinates: C, properties: P) -> Self {
        Self {
            coordinates,
            properties,
        }
    }

    /// Returns a reference to the properties.
    pub fn properties(&self) -> &P {
        &self.properties
    }

    /// Returns a mutable reference to the properties.
    pub fn properties_mut(&mut self) -> &mut P {
        &mut self.properties
    }

    /// Returns a reference to the coordinates.
    pub fn coordinates(&self) -> &C {
        &self.coordinates
    }

    /// Returns a mutable reference to the coordinates.
    pub fn coordinates_mut(&mut self) -> &mut C {
        &mut self.coordinates
    }

    /// Returns the [`MapFeatureRef`] wrapped [`self`], in order to serialize coordinates as a map
    pub fn as_map_feature(&self) -> MapFeatureRef<'_, C, P> {
        MapFeatureRef { inner: self }
    }

    /// Returns the [`MapFeature`] wrapped [`self`], in order to serialize coordinates as a map
    pub fn into_map_feature(self) -> MapFeature<C, P> {
        MapFeature { inner: self }
    }

    /// Converts the underlying `coordinates` into [`AnyCoordinate`].
    pub fn into_any_coordinates(self) -> Feature<AnyCoordinate, P>
    where
        C: Into<AnyCoordinate>,
    {
        Feature {
            coordinates: self.coordinates.into(),
            properties: self.properties,
        }
    }
}

impl<Pnt, Line, Poly, P> Feature<GenericCoordinate<Pnt, Line, Poly>, P> {
    /// If the inner [`GenericCoordinate`] is a [`Point`] variant, return the feature with that
    /// coordinate type.
    ///
    /// [`Point`]: [`GenericCoordinate::Point`]
    pub fn into_point_feature(self) -> Option<Feature<Pnt, P>> {
        self.coordinates
            .into_point()
            .map(|pnt| Feature::from_coords_and_properties(pnt, self.properties))
    }

    /// Shortcut to [`GenericCoordinate::is_point`].
    pub fn is_point_feature(&self) -> bool {
        self.coordinates.is_point()
    }

    /// If the inner [`GenericCoordinate`] is a [`LineString`] variant, return the feature with that
    /// coordinate type.
    ///
    /// [`LineString`]: [`GenericCoordinate::LineString`]
    pub fn into_line_string_feature(self) -> Option<Feature<Line, P>> {
        self.coordinates
            .into_line_string()
            .map(|pnt| Feature::from_coords_and_properties(pnt, self.properties))
    }

    /// Shortcut to [`GenericCoordinate::is_line_string`].
    pub fn is_line_string_feature(&self) -> bool {
        self.coordinates.is_line_string()
    }

    /// If the inner [`GenericCoordinate`] is a [`Polygon`] variant, return the feature with that
    /// coordinate type.
    ///
    /// [`Polygon`]: [`GenericCoordinate::Polygon`]
    pub fn into_polygon_feature(self) -> Option<Feature<Poly, P>> {
        self.coordinates
            .into_polygon()
            .map(|pnt| Feature::from_coords_and_properties(pnt, self.properties))
    }

    /// Shortcut to [`GenericCoordinate::is_polygon`].
    pub fn is_polygon_feature(&self) -> bool {
        self.coordinates.is_polygon()
    }
}

impl<C, P> Feature<Option<C>, P> {
    /// Takes the underlying coordinates out of the [`Feature`], if they haven't already been
    /// taken. This leaves [`None`] in its place.
    pub fn take_coordinates(&mut self) -> Option<C> {
        self.coordinates.take()
    }
}

impl<C, P> Feature<C, Option<P>> {
    /// Takes the underlying properties out of the [`Feature`], if they haven't already been
    /// taken. This leaves [`None`] in its place.
    pub fn take_properties(&mut self) -> Option<P> {
        self.properties.take()
    }
}

impl<P> Feature<(), P>
where
    P: Properties,
{
    /// Returns a builder, with an already known [`Properties::Id`].
    pub fn builder_with_args(args: P::RequiredArgs) -> FeatureBuilder<(), P> {
        FeatureBuilder::new_with_args(args)
    }
}

impl<C, P> Feature<C, P>
where
    C: Coordinate,
{
    /// Returns the geometry type for this [`Feature`].
    pub fn geometry_type(&self) -> GeometryType {
        self.coordinates.geometry_type()
    }
}

impl<C, P> Feature<C, P>
where
    P: Properties,
{
    /// Returns the Id from the underlying properties.
    pub fn id(&self) -> <P as Properties>::Id {
        self.properties.id()
    }
}

impl<C, P> Feature<C, P>
where
    for<'de> C: Deserialize<'de>,
    for<'de> P: Deserialize<'de>,
{
    pub fn from_deserializer<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(FeatureVisitor::new())
    }

    pub(crate) fn from_deserializer_with_path<'de, D>(
        deserializer: D,
    ) -> Result<Self, Error<D::Error>>
    where
        D: de::Deserializer<'de> + 'de,
        D::Error: 'static,
    {
        PathDeserializer::new(deserializer).deserialize_map(FeatureVisitor::new())
    }

    pub fn from_reader<R>(reader: R) -> Result<Self, serde_json::Error>
    where
        R: std::io::Read,
    {
        let mut deserializer = serde_json::Deserializer::from_reader(reader);
        Self::from_deserializer(&mut deserializer)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        Self::from_deserializer(&mut deserializer)
    }

    pub fn from_str(string: &str) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_str(string);
        Self::from_deserializer(&mut deserializer)
    }

    pub fn from_reader_with_path<R>(reader: R) -> Result<Self, Error<serde_json::Error>>
    where
        R: std::io::Read,
    {
        let mut deserializer = serde_json::Deserializer::from_reader(reader);
        Self::from_deserializer_with_path(&mut deserializer)
    }

    pub fn from_slice_with_path(bytes: &[u8]) -> Result<Self, Error<serde_json::Error>> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        Self::from_deserializer_with_path(&mut deserializer)
    }

    pub fn from_str_with_path(string: &str) -> Result<Self, Error<serde_json::Error>> {
        let mut deserializer = serde_json::Deserializer::from_str(string);
        Self::from_deserializer_with_path(&mut deserializer)
    }
}

impl<C, P> Serialize for Feature<C, P>
where
    C: Coordinate + Serialize,
    P: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(3))?;
        map_ser.serialize_entry("type", "Feature")?;
        map_ser.serialize_entry("geometry", &self.coordinates.as_geometry())?;
        map_ser.serialize_entry("properties", &self.properties)?;
        map_ser.end()
    }
}

impl<'de, C, P> Deserialize<'de> for Feature<C, P>
where
    C: Deserialize<'de>,
    P: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(FeatureVisitor::new())
    }
}

struct FeatureVisitor<C, P> {
    _marker: std::marker::PhantomData<(C, P)>,
}

impl<C, P> FeatureVisitor<C, P> {
    const fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, C, P> de::Visitor<'de> for FeatureVisitor<C, P>
where
    C: Deserialize<'de>,
    P: Deserialize<'de>,
{
    type Value = Feature<C, P>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a geojson feature")
    }

    #[inline]
    fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        FeatureDeserializer::new(&mut map_access).finish_deserializing()
    }
}

pub(crate) struct FeatureDeserializer<'a, C, P, M> {
    pub(crate) found_type: bool,
    pub(crate) geometry: Option<Geometry<C>>,
    pub(crate) properties: Option<P>,
    pub(crate) map_access: &'a mut M,
}

impl<'de, 'a, C, P, M> FeatureDeserializer<'a, C, P, M>
where
    C: Deserialize<'de>,
    P: Deserialize<'de>,
    M: de::MapAccess<'de>,
{
    pub fn new(map_access: &'a mut M) -> Self {
        Self {
            found_type: false,
            geometry: None,
            properties: None,
            map_access,
        }
    }

    pub(crate) fn finish_deserializing(mut self) -> Result<Feature<C, P>, M::Error> {
        #[derive(Debug)]
        enum FeatureFields {
            Type,
            Geometry,
            Properties,
            Unknown,
        }

        crate::util::impl_field_name_from_str! {
            FeatureFields {
                Type: b'T' | b't' => b"type",
                Geometry: b'G' | b'g' => b"eometry",
                Properties: b'P' | b'p' => b"roperties",
            }
        }

        crate::util::impl_str_marker_type!(FeatureMarker: "Feature");

        while let Some(field) = self
            .map_access
            .next_key_seed(FromStrVisitor::<FeatureFields>::new())?
        {
            let should_skip_remaining = match field {
                FeatureFields::Type => {
                    if self.found_type {
                        return Err(de::Error::duplicate_field("type"));
                    }

                    self.map_access.next_value::<FeatureMarker>()?;
                    self.found_type = true;
                    self.geometry.is_some() && self.properties.is_some()
                }
                FeatureFields::Geometry => {
                    if self
                        .geometry
                        .replace(self.map_access.next_value()?)
                        .is_some()
                    {
                        return Err(de::Error::duplicate_field("geometry"));
                    }
                    self.found_type && self.properties.is_some()
                }
                FeatureFields::Properties => {
                    if self
                        .properties
                        .replace(self.map_access.next_value()?)
                        .is_some()
                    {
                        return Err(de::Error::duplicate_field("properties"));
                    }
                    self.found_type && self.geometry.is_some()
                }
                FeatureFields::Unknown => {
                    // we still need to grab the next value, even if we have no idea what it is.
                    let _: de::IgnoredAny = self.map_access.next_value()?;
                    continue;
                }
            };

            // if we have the geometry, properties and found the 'type' field, we can't just
            // 'break', since the serde_json deserailizer errors out if the source isnt exhausted.
            if should_skip_remaining {
                crate::util::drain_map_access(self.map_access)?;
                break;
            }
        }

        let geometry = self.geometry.ok_or_else(|| de_err!(missing "geometry"))?;
        let properties = self
            .properties
            .ok_or_else(|| de_err!(missing "properties"))?;

        Ok(Feature {
            coordinates: geometry.into_coordinates(),
            properties,
        })
    }
}

impl<'a, C, P> Serialize for MapFeatureRef<'a, C, P>
where
    C: MapCoordinate + Serialize,
    P: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(3))?;
        map_ser.serialize_entry("type", "Feature")?;
        map_ser.serialize_entry("geometry", &self.inner.coordinates.as_map_geometry())?;
        map_ser.serialize_entry("properties", &self.inner.properties)?;
        map_ser.end()
    }
}

impl<C, P> Serialize for MapFeature<C, P>
where
    C: MapCoordinate + Serialize,
    P: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(3))?;
        map_ser.serialize_entry("type", "Feature")?;
        map_ser.serialize_entry("geometry", &self.inner.coordinates.as_map_geometry())?;
        map_ser.serialize_entry("properties", &self.inner.properties)?;
        map_ser.end()
    }
}

#[cfg(test)]
mod tests {
    use geo::Point;

    use super::*;

    #[test]
    fn test_case_insensive_deserialization() {
        let raw = serde_json::json!({
            "Type": "Feature",
            "GEOMETRY": {
                "tyPe": "Point",
                "cOOrdinAtes": [-123.456f64, 45.0f64]
            },
            "PrOpErTiEs": {
                "test-property": 5u32,
            }
        });

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub struct TestProps {
            test_property: u32,
        }

        let expected: Feature<Point, TestProps> = Feature {
            coordinates: Point::new_checked(-123.456, 45.0).unwrap(),
            properties: TestProps { test_property: 5 },
        };

        let deserialized: Feature<Point, TestProps> = serde_json::from_value(raw).unwrap();

        assert_eq!(deserialized, expected);
    }
}
