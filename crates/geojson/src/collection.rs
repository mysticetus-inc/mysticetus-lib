//! [`FeatureCollection`] definition + impls

use std::fmt;
use std::ops::{Deref, DerefMut};

use path_aware_serde::{Deserializer as PathDeserializer, Error};
use serde::ser::{self, SerializeMap, SerializeSeq};
use serde::{Deserializer, de};
use serde_helpers::from_str::FromStrVisitor;

use crate::Feature;
use crate::builder::FeatureCollectionBuilder;
use crate::geometry::any::{AnyCoordinate, GenericCoordinate};
use crate::geometry::{Coordinate, MapCoordinate};
use crate::macros::de_err;

/// A collection of GeoJson [`Feature`]'s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureCollection<C, P> {
    features: Vec<Feature<C, P>>,
}

/// A wrapper around a set of GeoJson [`Feature`]'s, that will serialize with map geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapFeatureCollection<'a, C, P> {
    features: MapFeatures<'a, C, P>,
}

/// A wrapper around a set of GeoJson [`Feature`]'s, that will serialize with map geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapFeatures<'a, C, P>(&'a Vec<Feature<C, P>>);

impl<C, P> FeatureCollection<C, P> {
    /// Builds a new, empty [`FeatureCollection`].
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    pub fn into_inner(self) -> Vec<Feature<C, P>> {
        self.features
    }

    /// Returns a [`FeatureCollectionBuilder`] for assembling a collection via a builder pattern.
    pub fn builder() -> FeatureCollectionBuilder<C, P> {
        FeatureCollectionBuilder::new()
    }

    /// Builds a new, empty [`FeatureCollection`], with room for 'capacity'
    /// features pre-allocated.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            features: Vec::with_capacity(capacity),
        }
    }

    /// Assembles a [`FeatureCollection`] from an existing [`Vec<Feature>`].
    pub fn from_features(features: Vec<Feature<C, P>>) -> Self {
        Self { features }
    }

    /// Returns wrapped set of features, that will serialize as a map.
    pub fn as_map_features(&self) -> MapFeatureCollection<'_, C, P> {
        MapFeatureCollection {
            features: MapFeatures(&self.features),
        }
    }
}

impl<C, P> FeatureCollection<C, P>
where
    C: Into<AnyCoordinate>,
{
    /// Converts all inner features into [`Feature<AnyCoordinate, P>`] and returns the resulting
    /// [`FeatureCollection`].
    pub fn into_any_coordinates(self) -> FeatureCollection<AnyCoordinate, P> {
        self.into_iter()
            .map(Feature::into_any_coordinates)
            .collect()
    }
}

impl<Pnt, Line, Poly, P> FeatureCollection<GenericCoordinate<Pnt, Line, Poly>, P> {
    /// Filters out all non-`Point` features, returning the resulting `Point`-only collection.
    pub fn into_point_collection(self) -> FeatureCollection<Pnt, P> {
        self.features
            .into_iter()
            .filter_map(Feature::into_point_feature)
            .collect()
    }

    /// Filters out all non-`LineString` features, returning the resulting `LineString`-only
    /// collection.
    pub fn into_line_string_collection(self) -> FeatureCollection<Line, P> {
        self.features
            .into_iter()
            .filter_map(Feature::into_line_string_feature)
            .collect()
    }

    /// Filters out all non-`Polygon` features, returning the resulting `Polygon`-only collection.
    pub fn into_polygon_collection(self) -> FeatureCollection<Poly, P> {
        self.features
            .into_iter()
            .filter_map(Feature::into_polygon_feature)
            .collect()
    }
}

// ------------ From Feature/Vec<Feature> impls ------------- //

impl<C, P> From<Feature<C, P>> for FeatureCollection<C, P> {
    fn from(feat: Feature<C, P>) -> Self {
        Self::from_features(vec![feat])
    }
}

impl<C, P> From<Vec<Feature<C, P>>> for FeatureCollection<C, P> {
    fn from(features: Vec<Feature<C, P>>) -> Self {
        Self::from_features(features)
    }
}

// ------------ Default impls ------------- //

impl<C, P> Default for FeatureCollection<C, P> {
    fn default() -> Self {
        Self::new()
    }
}

// ------------ Deref/DerefMut -> Vec<Feature> impls ------------- //

impl<C, P> Deref for FeatureCollection<C, P> {
    type Target = Vec<Feature<C, P>>;

    fn deref(&self) -> &Self::Target {
        &self.features
    }
}

impl<C, P> DerefMut for FeatureCollection<C, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.features
    }
}

// ---------- AsRef/AsMut -> &[Feature] impls ------------- //

impl<C, P> AsRef<[Feature<C, P>]> for FeatureCollection<C, P> {
    fn as_ref(&self) -> &[Feature<C, P>] {
        self.features.as_slice()
    }
}

impl<C, P> AsMut<[Feature<C, P>]> for FeatureCollection<C, P> {
    fn as_mut(&mut self) -> &mut [Feature<C, P>] {
        self.features.as_mut_slice()
    }
}

// ------------ From/IntoIterator impls ----------------- //

impl<C, P> IntoIterator for FeatureCollection<C, P> {
    type Item = Feature<C, P>;
    type IntoIter = std::vec::IntoIter<Feature<C, P>>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.into_iter()
    }
}

impl<'a, C, P> IntoIterator for &'a FeatureCollection<C, P> {
    type Item = &'a Feature<C, P>;
    type IntoIter = std::slice::Iter<'a, Feature<C, P>>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.iter()
    }
}

impl<'a, C, P> IntoIterator for &'a mut FeatureCollection<C, P> {
    type Item = &'a mut Feature<C, P>;
    type IntoIter = std::slice::IterMut<'a, Feature<C, P>>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.iter_mut()
    }
}

impl<C, P> FromIterator<Feature<C, P>> for FeatureCollection<C, P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Feature<C, P>>,
    {
        let features = iter.into_iter().collect::<Vec<Feature<C, P>>>();

        Self { features }
    }
}

// ----------- Serialize/Deserialize impls ----------------- //

impl<C, P> ser::Serialize for FeatureCollection<C, P>
where
    C: Coordinate + ser::Serialize,
    P: ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(2))?;
        map_ser.serialize_entry("type", "FeatureCollection")?;
        map_ser.serialize_entry("features", &self.features)?;
        map_ser.end()
    }
}

impl<'de, C, P> de::Deserialize<'de> for FeatureCollection<C, P>
where
    C: de::Deserialize<'de> + 'de,
    P: de::Deserialize<'de> + 'de,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(FeatureCollectionVisitor::new())
    }
}

impl<'de, C, P> FeatureCollection<C, P>
where
    C: de::Deserialize<'de> + 'de,
    P: de::Deserialize<'de> + 'de,
{
    /// Deserialize with a given deserializer, but wrap with a path-aware wrapper to improve
    /// errors.
    pub fn deserialize_with_path_errors<D>(deserializer: D) -> Result<Self, Error<D::Error>>
    where
        D: de::Deserializer<'de> + 'de,
        D::Error: 'static,
    {
        PathDeserializer::new(deserializer).deserialize_map(FeatureCollectionVisitor::new())
    }
}

struct FeatureCollectionVisitor<C, P> {
    _marker: std::marker::PhantomData<(C, P)>,
}

impl<C, P> FeatureCollectionVisitor<C, P> {
    fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, C, P> de::Visitor<'de> for FeatureCollectionVisitor<C, P>
where
    C: de::Deserialize<'de> + 'de,
    P: de::Deserialize<'de> + 'de,
{
    type Value = FeatureCollection<C, P>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a geojson feature collection")
    }

    fn visit_seq<S>(self, mut seq_access: S) -> Result<Self::Value, S::Error>
    where
        S: de::SeqAccess<'de>,
    {
        let mut collection = seq_access
            .size_hint()
            .map(FeatureCollection::with_capacity)
            .unwrap_or_default();

        while let Some(feature) = seq_access.next_element()? {
            collection.push(feature);
        }

        Ok(collection)
    }

    fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        FeatureCollectionDeserializer::new(&mut map_access).finish_deserializing()
    }
}

pub(crate) struct FeatureCollectionDeserializer<'a, C, P, M> {
    pub(crate) found_type: bool,
    pub(crate) features: Option<Vec<Feature<C, P>>>,
    pub(crate) map_access: &'a mut M,
}

impl<'de, 'a, C, P, M> FeatureCollectionDeserializer<'a, C, P, M>
where
    C: serde::Deserialize<'de>,
    P: serde::Deserialize<'de>,
    M: de::MapAccess<'de>,
{
    pub(crate) fn new(map_access: &'a mut M) -> Self {
        Self {
            map_access,
            found_type: false,
            features: None,
        }
    }

    pub(crate) fn finish_deserializing(mut self) -> Result<FeatureCollection<C, P>, M::Error> {
        enum FeatureCollectionField {
            Type,
            Features,
            Unknown,
        }

        crate::util::impl_field_name_from_str! {
            FeatureCollectionField {
                Type: b'T' | b't' => b"ype",
                Features: b'F' | b'f' => b"eatures",
            }
        }

        crate::util::impl_str_marker_type!(FeatureCollectionMarker: "FeatureCollection");

        while let Some(field) = self
            .map_access
            .next_key_seed(FromStrVisitor::<FeatureCollectionField>::new())?
        {
            let should_skip_remaining = match field {
                FeatureCollectionField::Type => {
                    if self.found_type {
                        return Err(de::Error::duplicate_field("type"));
                    }

                    self.map_access.next_value::<FeatureCollectionMarker>()?;
                    self.found_type = true;
                    self.features.is_some()
                }
                FeatureCollectionField::Features => {
                    if self
                        .features
                        .replace(self.map_access.next_value()?)
                        .is_some()
                    {
                        return Err(de::Error::duplicate_field("features"));
                    }
                    self.found_type
                }
                _ => {
                    // we still need to drain the value even if we dont care about it.
                    let _: de::IgnoredAny = self.map_access.next_value()?;
                    continue;
                }
            };

            if should_skip_remaining {
                crate::util::drain_map_access(self.map_access)?;
                break;
            }
        }

        let features = self.features.ok_or_else(|| de_err!(missing "features"))?;

        Ok(FeatureCollection { features })
    }
}

impl<'a, C, P> ser::Serialize for MapFeatureCollection<'a, C, P>
where
    C: MapCoordinate + ser::Serialize,
    P: ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(2))?;
        map_ser.serialize_entry("type", "FeatureCollection")?;
        map_ser.serialize_entry("features", &self.features)?;
        map_ser.end()
    }
}

impl<'a, C, P> ser::Serialize for MapFeatures<'a, C, P>
where
    C: MapCoordinate + ser::Serialize,
    P: ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut seq_ser = serializer.serialize_seq(Some(self.0.len()))?;

        for feature in self.0.iter() {
            seq_ser.serialize_element(&feature.as_map_feature())?;
        }

        seq_ser.end()
    }
}
