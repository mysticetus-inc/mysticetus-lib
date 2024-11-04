//! [`FeatureCollectionBuilder`] definition + impls

use super::FeatureBuilder;
use crate::geometry::{AnyCoordinate, Coordinate};
use crate::properties::{DisplayProps, Properties, PropertyMap, TimedProps};
use crate::{Feature, FeatureCollection};

/// Helper for building a feature collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureCollectionBuilder<C, P> {
    features: Vec<Feature<C, P>>,
}

/// [`FeatureCollection`] analog to [`FeatureBuilder`].
///
/// Instead of [`FeatureBuilder::build`], this holds onto a mutable reference to the parent
/// [`FeatureCollectionBuilder`], and builds + inserts the feature when
/// [`NewFeatureBuilder::insert_feature`] is called.
#[derive(Debug, PartialEq, Eq)]
pub struct NewFeatureBuilder<'a, ParentG, ParentP, NewG = (), NewP = ()> {
    collection_builder: &'a mut FeatureCollectionBuilder<ParentG, ParentP>,
    new: FeatureBuilder<NewG, NewP>,
}

impl<C, P> FeatureCollectionBuilder<C, P> {
    /// Instantiates an empty feature collection builder.
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    /// Instantiates an empty feature collection builder, but with enough pre-allocated space to
    /// contain 'capactiy' features.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            features: Vec::with_capacity(capacity),
        }
    }
}

impl<C, P> Default for FeatureCollectionBuilder<C, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, P> FeatureCollectionBuilder<C, P> {
    /// Returns a [`NewFeatureBuilder`] to build a feature that'll end up in the
    /// [`FeatureCollection`].
    ///
    /// The internal [`Feature`] starts out empty, with the coordinates/properties == [`None`]
    pub fn new_empty_feature(&mut self) -> NewFeatureBuilder<'_, C, P, (), ()> {
        NewFeatureBuilder::new(self, FeatureBuilder::empty())
    }

    /// Returns a [`NewFeatureBuilder`] with the give coordinates already set.
    pub fn new_feature_with_coords<NC>(
        &mut self,
        coords: NC,
    ) -> NewFeatureBuilder<'_, C, P, NC, ()> {
        NewFeatureBuilder::new(self, FeatureBuilder::from_coordinates(coords))
    }

    /// Returns a [`NewFeatureBuilder`] with the given properties already set.
    pub fn new_feature_with_props<NC, NP>(
        &mut self,
        props: NP,
    ) -> NewFeatureBuilder<'_, C, P, (), NP> {
        NewFeatureBuilder::new(self, FeatureBuilder::from_properties(props))
    }

    /// Inserts a feature that may or may not have been built manually.
    pub fn insert_feature(&mut self, feature: Feature<C, P>) {
        self.features.push(feature);
    }
}

impl<C, P> FeatureCollectionBuilder<C, P>
where
    P: Default,
{
    /// Returns a [`NewFeatureBuilder`] to build a feature with an already specified Id that'll
    /// end up in the [`FeatureCollection`].
    pub fn new_feature_with_default_props<NC>(&mut self) -> NewFeatureBuilder<'_, C, P, (), P> {
        NewFeatureBuilder::new(self, FeatureBuilder::with_default_properties())
    }
}

impl<C, P> FeatureCollectionBuilder<C, P>
where
    P: Properties,
{
    /// Returns a [`NewFeatureBuilder`] to build a feature with an already specified Id that'll
    /// end up in the [`FeatureCollection`].
    pub fn new_feature_with_args(
        &mut self,
        args: P::RequiredArgs,
    ) -> NewFeatureBuilder<'_, C, P, (), P> {
        NewFeatureBuilder::new_with_args(self, args)
    }
}

impl<C, P> FeatureCollectionBuilder<C, P>
where
    C: Coordinate,
    P: Properties,
{
    /// Builds the resulting [`FeatureCollection`].
    pub fn build(self) -> FeatureCollection<C, P> {
        self.features.into()
    }
}

impl<'a, PG, PP, NG, NP> NewFeatureBuilder<'a, PG, PP, NG, NP> {
    /// Internal method called by [`FeatureCollectionBuilder::new_feature`]
    fn new(
        collection_builder: &'a mut FeatureCollectionBuilder<PG, PP>,
        feature_builder: FeatureBuilder<NG, NP>,
    ) -> NewFeatureBuilder<'a, PG, PP, NG, NP> {
        NewFeatureBuilder {
            collection_builder,
            new: feature_builder,
        }
    }
}

impl<'a, PG, PP, NP> NewFeatureBuilder<'a, PG, PP, (), NP>
where
    NP: Properties,
{
    /// Internal method called by [`FeatureCollectionBuilder::new_feature_with_id`]
    fn new_with_args(
        collection_builder: &'a mut FeatureCollectionBuilder<PG, PP>,
        args: NP::RequiredArgs,
    ) -> NewFeatureBuilder<'a, PG, PP, (), NP> {
        NewFeatureBuilder {
            collection_builder,
            new: FeatureBuilder::new_with_args(args),
        }
    }
}

impl<'a, PG, PP, NP> NewFeatureBuilder<'a, PG, PP, (), NP> {
    /// Adds coordintes to the underlying [`Feature`].
    pub fn coordintes<NG>(self, coordinates: NG) -> NewFeatureBuilder<'a, PG, PP, NG, NP> {
        NewFeatureBuilder {
            collection_builder: self.collection_builder,
            new: self.new.with_coordinates(coordinates),
        }
    }
}

impl<'a, PG, PP, NG, NP> NewFeatureBuilder<'a, PG, PP, NG, NP>
where
    NG: Into<AnyCoordinate>,
{
    /// Converts the underlying coordinates into [`AnyCoordinate`].
    #[allow(clippy::wrong_self_convention)]
    pub fn as_any_coordinate(self) -> NewFeatureBuilder<'a, PG, PP, AnyCoordinate, NP> {
        NewFeatureBuilder {
            collection_builder: self.collection_builder,
            new: self.new.as_any_coordinate(),
        }
    }
}

impl<'a, PG, PP, NG, NP> NewFeatureBuilder<'a, PG, PP, NG, NP>
where
    NP: TimedProps,
{
    /// Sets the timestamp on the underlying type that implements [`Properties`].
    pub fn set_timestamp(self, timestamp: timestamp::Timestamp) -> Self {
        Self {
            new: self.new.set_timestamp(timestamp),
            collection_builder: self.collection_builder,
        }
    }
}

impl<'a, PG, PP, NG, NP> NewFeatureBuilder<'a, PG, PP, NG, NP>
where
    NP: Properties,
{
    /// Inserts a property into the underlying [`Properties`] map.
    pub fn insert_property<S>(self, key: S, value: NP::Value) -> Self
    where
        S: Into<String>,
    {
        Self {
            new: self.new.insert_property(key, value),
            collection_builder: self.collection_builder,
        }
    }

    /// Takes an iterator of key/value pairs, and inserts them into the underlying
    /// [`Properties`] map.
    pub fn extend_properties<I, S, V>(self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<NP::Value>,
    {
        Self {
            new: self.new.extend_properties(iter),
            collection_builder: self.collection_builder,
        }
    }
}

impl<'a, PG, PP, NG, NP> NewFeatureBuilder<'a, PG, PP, NG, NP>
where
    NP: DisplayProps,
{
    /// Inserts a display property into the underlying [`DisplayProps`] map.
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    pub fn insert_display_property<S>(
        self,
        key: S,
        value: <NP::DisplayProps as PropertyMap>::Value,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            new: self.new.insert_display_property(key, value),
            collection_builder: self.collection_builder,
        }
    }

    /// Takes an iterator of key/value pairs, and inserts them into the underlying
    /// [`DisplayProps`] map.
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    pub fn extend_display_properties<I, S, V>(self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<<NP::DisplayProps as PropertyMap>::Value>,
    {
        Self {
            new: self.new.extend_display_properties(iter),
            collection_builder: self.collection_builder,
        }
    }
}

impl<'a, C, P> NewFeatureBuilder<'a, C, P, C, P>
where
    C: Coordinate,
    P: Properties,
{
    /// Consumes this builder, inserting the assembled [`Feature`] into the parent
    /// [`FeatureCollectionBuilder`]
    pub fn insert_feature(self) {
        self.collection_builder.features.push(self.new.build());
    }
}
