//! [`GeoJson`] enum, that can contain either a [`Feature`] or [`FeatureCollection`]

use std::fmt;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

use path_aware_serde::{Deserializer as PathDeserializer, Error};
use serde::de::{self, Deserializer};
use serde::ser;

use super::geometry::{Coordinate, MapCoordinate};
use super::{Feature, FeatureCollection};

/// Valid GeoJson, either as a single [`Feature`], or [`FeatureCollection`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeoJson<C, P> {
    Feature(Feature<C, P>),
    FeatureCollection(FeatureCollection<C, P>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapGeoJson<'a, C, P> {
    inner: &'a GeoJson<C, P>,
}

impl<C, P> GeoJson<C, P> {
    /// Turns the [`GeoJson`] into a [`FeatureCollection`].
    ///
    /// If the variant of 'self' is [`GeoJson::Feature`], a new [`FeatureCollection`] containing
    /// only that [`Feature`] is created.
    pub fn into_feature_collection(self) -> FeatureCollection<C, P> {
        match self {
            Self::Feature(feature) => FeatureCollection::from_features(vec![feature]),
            Self::FeatureCollection(collec) => collec,
        }
    }

    /// Returns the underlying [`FeatureCollection`], converting self to a [`FeatureCollection`]
    /// first if 'self' started as a [`GeoJson::Feature`] variant.
    ///
    /// ```
    /// # use geojson::{GeoJson, Feature, FeatureCollection};
    /// # use geojson::geometry::Point;
    /// # use geojson::properties::base_props::BaseProperties;
    /// # use timestamp::Timestamp;
    /// # type Props = BaseProperties<usize, Option<Timestamp>, Option<String>, String>;
    ///
    /// let feature: Feature<Point, Props> = // ...
    /// #   Feature::builder()
    /// #       .with_coordinates(Point::new_checked(0.0, 0.0).unwrap())
    /// #       .with_properties(Props::default())
    /// #       .build();
    /// #
    /// #
    /// #
    ///
    /// let mut geojson = GeoJson::Feature(feature);
    ///
    /// let collec: &mut FeatureCollection<Point, Props> = geojson.to_feature_collection_mut();
    ///
    /// assert_eq!(collec.len(), 1);
    /// ```
    pub fn to_feature_collection_mut(&mut self) -> &mut FeatureCollection<C, P> {
        match self {
            Self::FeatureCollection(collec) => collec,
            Self::Feature(_) => {
                let old = std::mem::replace(
                    self,
                    GeoJson::FeatureCollection(FeatureCollection::with_capacity(8)),
                );

                let collec = match self {
                    Self::FeatureCollection(collec) => collec,
                    Self::Feature(_) => unreachable!(),
                };

                match old {
                    Self::Feature(feature) => collec.push(feature),
                    Self::FeatureCollection(_) => unreachable!(),
                }

                collec
            }
        }
    }

    /// Returns a wrapped [`GeoJson`] that will serialize coordinates as a map.
    pub fn as_map_geojson(&self) -> MapGeoJson<'_, C, P> {
        MapGeoJson { inner: self }
    }

    /// Returns the number of [`Feature`]'s contained in this [`GeoJson`].
    ///
    /// Returns 1 if the variant of 'self' is [`GeoJson::Feature`].
    pub fn len(&self) -> usize {
        match self {
            Self::Feature(_) => 1,
            Self::FeatureCollection(collec) => collec.len(),
        }
    }

    /// Chekcs if the underlying variant of 'self' is [`GeoJson::FeatureCollection`], and if it's
    /// empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Feature(_) => false,
            Self::FeatureCollection(collec) => collec.is_empty(),
        }
    }

    /// Returns a reference to the first [`Feature`] in this [`GeoJson`].
    ///
    /// Only returns [`None`] if the variant of 'self' is [`GeoJson::FeatureCollection`], and
    /// if it's an empty collection.
    pub fn first(&self) -> Option<&Feature<C, P>> {
        match self {
            Self::Feature(feat) => Some(feat),
            Self::FeatureCollection(collec) => collec.first(),
        }
    }

    /// Returns an iterator over references to the features inside. If this is a single feature,
    /// the iterator will only yield one item.
    pub fn iter(&self) -> GeoJsonIter<'_, C, P> {
        let inner = match self {
            Self::Feature(feat) => InnerGeoJsonIter::Feature(Some(feat)),
            Self::FeatureCollection(collec) => InnerGeoJsonIter::CollecIter(collec.iter()),
        };

        GeoJsonIter { inner }
    }

    /// Returns an iterator over mutable references to the features inside. If this is a single
    /// feature, the iterator will only yield one item.
    pub fn iter_mut(&mut self) -> GeoJsonIterMut<'_, C, P> {
        let inner = match self {
            Self::Feature(feat) => InnerGeoJsonIter::Feature(Some(feat)),
            Self::FeatureCollection(collec) => InnerGeoJsonIter::CollecIter(collec.iter_mut()),
        };

        GeoJsonIterMut { inner }
    }
}

impl<'a, C, P> IntoIterator for &'a GeoJson<C, P> {
    type Item = &'a Feature<C, P>;
    type IntoIter = GeoJsonIter<'a, C, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, C, P> IntoIterator for &'a mut GeoJson<C, P> {
    type Item = &'a mut Feature<C, P>;
    type IntoIter = GeoJsonIterMut<'a, C, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<C, P> FromIterator<Feature<C, P>> for GeoJson<C, P> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Feature<C, P>>,
    {
        let mut iter = iter.into_iter();

        let first_feat = match iter.next() {
            Some(feat) => feat,
            None => return GeoJson::FeatureCollection(Default::default()),
        };

        let mut features = match iter.next() {
            Some(next_feat) => vec![first_feat, next_feat],
            None => return GeoJson::Feature(first_feat),
        };

        features.extend(iter);

        GeoJson::FeatureCollection(features.into())
    }
}

impl<C, P> IntoIterator for GeoJson<C, P> {
    type Item = Feature<C, P>;
    type IntoIter = IntoGeoJsonIter<C, P>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = match self {
            Self::Feature(feat) => InnerGeoJsonIter::Feature(Some(feat)),
            Self::FeatureCollection(collec) => InnerGeoJsonIter::CollecIter(collec.into_iter()),
        };

        IntoGeoJsonIter { inner }
    }
}

impl<C, P> From<Feature<C, P>> for GeoJson<C, P> {
    fn from(feature: Feature<C, P>) -> Self {
        Self::Feature(feature)
    }
}

impl<C, P> From<FeatureCollection<C, P>> for GeoJson<C, P> {
    fn from(collec: FeatureCollection<C, P>) -> Self {
        Self::FeatureCollection(collec)
    }
}

impl<C, P> From<Vec<Feature<C, P>>> for GeoJson<C, P> {
    fn from(features: Vec<Feature<C, P>>) -> Self {
        Self::FeatureCollection(FeatureCollection::from_features(features))
    }
}

impl<C, P> GeoJson<C, P>
where
    for<'de> C: de::Deserialize<'de>,
    for<'de> P: de::Deserialize<'de>,
{
    pub fn from_deserializer<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(GeoJsonVisitor::new())
    }

    pub fn deserialize_with_path<'de, D>(deserializer: D) -> Result<Self, Error<D::Error>>
    where
        D: de::Deserializer<'de> + 'de,
        D::Error: 'static,
    {
        PathDeserializer::new(deserializer).deserialize_map(GeoJsonVisitor::new())
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

    pub fn from_json_str(string: &str) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_str(string);
        Self::from_deserializer(&mut deserializer)
    }

    pub fn from_reader_with_path<R>(reader: R) -> Result<Self, Error<serde_json::Error>>
    where
        R: std::io::Read,
    {
        let mut deserializer = serde_json::Deserializer::from_reader(reader);
        Self::deserialize_with_path(&mut deserializer)
    }

    pub fn from_slice_with_path(bytes: &[u8]) -> Result<Self, Error<serde_json::Error>> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        Self::deserialize_with_path(&mut deserializer)
    }

    pub fn from_str_with_path(string: &str) -> Result<Self, Error<serde_json::Error>> {
        let mut deserializer = serde_json::Deserializer::from_str(string);
        Self::deserialize_with_path(&mut deserializer)
    }
}

impl<C, P> ser::Serialize for GeoJson<C, P>
where
    C: Coordinate + ser::Serialize,
    P: ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Self::Feature(feature) => feature.serialize(serializer),
            Self::FeatureCollection(collection) => collection.serialize(serializer),
        }
    }
}

impl<'de, C, P> de::Deserialize<'de> for GeoJson<C, P>
where
    C: de::Deserialize<'de>,
    P: de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(GeoJsonVisitor::new())
    }
}

#[derive(Default)]
struct GeoJsonVisitor<C, P> {
    _marker: std::marker::PhantomData<(C, P)>,
}

impl<C, P> GeoJsonVisitor<C, P> {
    fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, C, P> de::Visitor<'de> for GeoJsonVisitor<C, P>
where
    C: de::Deserialize<'de>,
    P: de::Deserialize<'de>,
{
    type Value = GeoJson<C, P>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "either a GeoJson feature, or a feature collection"
        )
    }

    fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        de_impl::GeoJsonDeserializer::new(&mut map_access).finish_deserializing()
    }
}

impl<'a, C, P> ser::Serialize for MapGeoJson<'a, C, P>
where
    C: MapCoordinate + ser::Serialize,
    P: ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self.inner {
            GeoJson::Feature(feature) => feature.as_map_feature().serialize(serializer),
            GeoJson::FeatureCollection(collec) => collec.as_map_features().serialize(serializer),
        }
    }
}

/// inner helper for iterating over geojson. This provides all iterator functionality
/// for the by-ref, by-mut-ref and owned iterators, while also not exposing an enum.
enum InnerGeoJsonIter<Feat, CollecIter> {
    Feature(Option<Feat>),
    CollecIter(CollecIter),
}

impl<F, C> Iterator for InnerGeoJsonIter<F, C>
where
    C: Iterator<Item = F>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Feature(feat_opt) => feat_opt.take(),
            Self::CollecIter(iter) => iter.next(),
        }
    }
}

impl<F, C> DoubleEndedIterator for InnerGeoJsonIter<F, C>
where
    C: Iterator<Item = F> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Feature(feat_opt) => feat_opt.take(),
            Self::CollecIter(iter) => iter.next_back(),
        }
    }
}

impl<F, C> InnerGeoJsonIter<F, C>
where
    C: ExactSizeIterator,
{
    fn len(&self) -> usize {
        match self {
            Self::Feature(Some(_)) => 1,
            Self::Feature(None) => 0,
            Self::CollecIter(iter) => iter.len(),
        }
    }
}

pub struct GeoJsonIter<'a, C, P> {
    inner: InnerGeoJsonIter<&'a Feature<C, P>, Iter<'a, Feature<C, P>>>,
}

pub struct GeoJsonIterMut<'a, C, P> {
    inner: InnerGeoJsonIter<&'a mut Feature<C, P>, IterMut<'a, Feature<C, P>>>,
}

pub struct IntoGeoJsonIter<C, P> {
    inner: InnerGeoJsonIter<Feature<C, P>, IntoIter<Feature<C, P>>>,
}

impl<'a, C, P> Iterator for GeoJsonIter<'a, C, P> {
    type Item = &'a Feature<C, P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, C, P> DoubleEndedIterator for GeoJsonIter<'a, C, P> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, C, P> ExactSizeIterator for GeoJsonIter<'a, C, P> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, C, P> Iterator for GeoJsonIterMut<'a, C, P> {
    type Item = &'a mut Feature<C, P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, C, P> DoubleEndedIterator for GeoJsonIterMut<'a, C, P> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, C, P> ExactSizeIterator for GeoJsonIterMut<'a, C, P> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<C, P> Iterator for IntoGeoJsonIter<C, P> {
    type Item = Feature<C, P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<C, P> DoubleEndedIterator for IntoGeoJsonIter<C, P> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<C, P> ExactSizeIterator for IntoGeoJsonIter<C, P> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub(crate) mod de_impl {
    use std::marker::PhantomData;

    use serde::{Deserialize, de};
    use serde_helpers::from_str::FromStrVisitor;

    use crate::collection::FeatureCollectionDeserializer;
    use crate::feature::FeatureDeserializer;
    use crate::geometry::Geometry;
    use crate::{Feature, GeoJson};

    enum Field {
        Type,
        Features,
        Geometry,
        Properties,
        Unknown,
    }

    crate::util::impl_field_name_from_str! {
        Field {
            Type: b'T' | b't' => b"ype",
            Features: b'F' | b'f' => b"eatures",
            Geometry: b'G' | b'g' => b"eometry",
            Properties: b'P' | b'p' => b"roperties",
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
    enum Type {
        Feature,
        FeatureCollection,
    }

    pub(crate) struct GeoJsonDeserializer<'a, C, P, M> {
        map_access: &'a mut M,
        _marker: PhantomData<fn(C, P)>,
    }

    impl<'de, 'a, C, P, M> GeoJsonDeserializer<'a, C, P, M>
    where
        C: Deserialize<'de>,
        P: Deserialize<'de>,
        M: de::MapAccess<'de>,
    {
        pub(crate) fn new(map_access: &'a mut M) -> Self {
            Self {
                map_access,
                _marker: PhantomData,
            }
        }

        fn finish_as_feature(
            self,
            found_type: bool,
            geometry: Option<Geometry<C>>,
            properties: Option<P>,
        ) -> Result<GeoJson<C, P>, M::Error> {
            let de = FeatureDeserializer {
                found_type,
                geometry,
                properties,
                map_access: self.map_access,
            };

            de.finish_deserializing().map(GeoJson::Feature)
        }

        fn finish_as_feature_collection(
            self,
            found_type: bool,
            features: Option<Vec<Feature<C, P>>>,
        ) -> Result<GeoJson<C, P>, M::Error> {
            let de = FeatureCollectionDeserializer {
                found_type,
                features,
                map_access: self.map_access,
            };

            de.finish_deserializing().map(GeoJson::FeatureCollection)
        }

        pub(crate) fn finish_deserializing(self) -> Result<GeoJson<C, P>, M::Error> {
            // use the first non-unknown key to infer the expected type
            while let Some(field) = self
                .map_access
                .next_key_seed(FromStrVisitor::<Field>::new())?
            {
                match field {
                    Field::Type => match self.map_access.next_value::<Type>()? {
                        Type::Feature => return self.finish_as_feature(true, None, None),
                        Type::FeatureCollection => {
                            return self.finish_as_feature_collection(true, None);
                        }
                    },
                    Field::Features => {
                        let features: Vec<Feature<C, P>> = self.map_access.next_value()?;
                        return self.finish_as_feature_collection(false, Some(features));
                    }
                    Field::Geometry => {
                        let geometry: Geometry<C> = self.map_access.next_value()?;
                        return self.finish_as_feature(false, Some(geometry), None);
                    }
                    Field::Properties => {
                        let properties: P = self.map_access.next_value()?;
                        return self.finish_as_feature(false, None, Some(properties));
                    }
                    Field::Unknown => _ = self.map_access.next_value::<de::IgnoredAny>()?,
                }
            }

            // if we didn't find any fields that we need (and return in the loop), use a missing
            // 'type' error
            Err(de::Error::missing_field("type"))
        }
    }
}
