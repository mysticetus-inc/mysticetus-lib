//! [`FeatureBuilder`] definition + impls.

// use super::CmdCenterPropsBuilder;

use super::properties::BasePropertiesBuilder;
use crate::Feature;
use crate::geometry::{AnyCoordinate, Coordinate, LineString, Point, Polygon};
use crate::properties::base_props::{BaseProperties, Requirement};
use crate::properties::{DisplayProps, Properties, PropertyMap, TimedProps};

/// A builder-pattern struct for assembling valid GeoJson '[`Feature`]s'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureBuilder<C = (), P = ()> {
    coordinates: C,
    properties: P,
}

impl<C, P> FeatureBuilder<C, P>
where
    C: Coordinate,
    P: Properties,
{
    /// Assembles the [`Feature`] and returns it.
    pub fn build(self) -> Feature<C, P> {
        Feature::from_coords_and_properties(self.coordinates, self.properties)
    }
}

impl FeatureBuilder<(), ()> {
    /// Assembles an empty [`FeatureBuilder`], defaulting to [`None`] for both coordinates
    /// and properties.
    ///
    /// The [`Default`] impl returns this.
    pub fn empty() -> Self {
        FeatureBuilder {
            coordinates: (),
            properties: (),
        }
    }
}

impl<C> FeatureBuilder<C, ()> {
    /// Instantiates a [`FeatureBuilder`] with already defined coordinates.
    pub fn from_coordinates(coordinates: C) -> Self {
        FeatureBuilder {
            coordinates,
            properties: (),
        }
    }
}

impl<P> FeatureBuilder<(), P> {
    /// Instantiates a [`FeatureBuilder`] with already defined properties.
    pub fn from_properties(properties: P) -> Self {
        FeatureBuilder {
            coordinates: (),
            properties,
        }
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Inserts the defined coordintes.
    pub fn with_coordinates<C2>(self, coordinates: C2) -> FeatureBuilder<C2, P> {
        FeatureBuilder {
            coordinates,
            properties: self.properties,
        }
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Inserts a [`Point`], using any of its [`From`] impls.
    pub fn point_from<PT>(self, point: PT) -> FeatureBuilder<Point, P>
    where
        PT: Into<Point>,
    {
        self.with_coordinates(point.into())
    }

    /// Attempts to insert a [`Point`], using any of its [`TryFrom`] impls.
    pub fn point_try_from<PT>(
        self,
        point: PT,
    ) -> Result<FeatureBuilder<Point, P>, <PT as TryInto<Point>>::Error>
    where
        PT: TryInto<Point>,
    {
        let point = point.try_into()?;
        Ok(self.with_coordinates(point))
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Inserts a [`LineString`] from any type that implements [`Into<LineString>`].
    pub fn line_string_from<LS>(self, line_string: LS) -> FeatureBuilder<LineString, P>
    where
        LS: Into<LineString>,
    {
        self.with_coordinates(line_string.into())
    }

    /// Inserts a [`LineString`] from an iterator of [`Point`]s. Use [`line_string_from`] if the
    /// type being passed in implements [`Into<LineString>`].
    ///
    /// [`line_string_from`]: [`Self::line_string_from`]
    pub fn line_string_from_iter<Iter>(self, iter: Iter) -> FeatureBuilder<LineString, P>
    where
        Iter: IntoIterator<Item = Point>,
    {
        self.with_coordinates(iter.into_iter().collect())
    }

    /// Attempts to insert a [`LineString`], taking an iterator of items that implement
    /// [`TryInto<Point>`].
    pub fn line_string_try_from<Iter, Inner>(
        self,
        iter: Iter,
    ) -> Result<FeatureBuilder<LineString, P>, <Inner as TryInto<Point>>::Error>
    where
        Iter: IntoIterator<Item = Inner>,
        Inner: TryInto<Point>,
    {
        let line_string = iter
            .into_iter()
            .map(|inner| inner.try_into())
            .collect::<Result<Vec<Point>, _>>()?;

        Ok(self.with_coordinates(line_string.into()))
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Inserts a [`Polygon`] from any type that implements [`Into<Polygon>`].
    pub fn polygon_from<Poly>(self, polygon: Poly) -> FeatureBuilder<Polygon, P>
    where
        Poly: Into<Polygon>,
    {
        self.with_coordinates(polygon.into())
    }

    /// Inserts a [`Polygon`] from nested iterators, where the inner item are [`Point`]s. Use
    /// [`polygon_from`] if the type being passed in implements [`Into<Polygon>`].
    ///
    /// [`polygon_from`]: [`Self::polygon_from`]
    pub fn polygon_from_iter<Iter, InnerIter>(self, iter: Iter) -> FeatureBuilder<Polygon, P>
    where
        Iter: IntoIterator<Item = InnerIter>,
        InnerIter: IntoIterator<Item = Point>,
    {
        let polygon = iter
            .into_iter()
            .map(|inner| inner.into_iter().collect::<LineString>())
            .collect::<Polygon>();

        self.with_coordinates(polygon)
    }

    /// Attempts to insert a [`Polygon`] from nested iterators, where the inner item implements
    /// [`TryInto<Point>`].
    pub fn polygon_try_from<Iter, InnerIter, Inner>(
        self,
        iter: Iter,
    ) -> Result<FeatureBuilder<Polygon, P>, <Inner as TryInto<Point>>::Error>
    where
        Iter: IntoIterator<Item = InnerIter>,
        InnerIter: IntoIterator<Item = Inner>,
        Inner: TryInto<Point>,
    {
        let polygon = iter
            .into_iter()
            .map(|inner| {
                inner
                    .into_iter()
                    .map(|inner| inner.try_into())
                    .collect::<Result<Vec<Point>, _>>()
                    .map(|line_string| line_string.into())
            })
            .collect::<Result<Vec<LineString>, _>>()?;

        Ok(self.with_coordinates(polygon.into()))
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Inserts a new property map.
    pub fn with_properties<P2>(self, properties: P2) -> FeatureBuilder<C, P2> {
        FeatureBuilder {
            coordinates: self.coordinates,
            properties,
        }
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Passes in an empty builder for [`CmdCenterProps`] to the function argument, and
    /// uses its return value as the new properties.
    ///
    /// [`CmdCenterProps`]: [`crate::properties::cmd_center_props::CmdCenterProps`]
    pub fn with_base_prop_builder<F, Id, Ts, Name, Val>(
        self,
        builder_fn: F,
    ) -> FeatureBuilder<C, BaseProperties<Id, Ts, Name, Val>>
    where
        F: FnOnce(
            BasePropertiesBuilder<(), Option<timestamp::Timestamp>, Option<String>, Val>,
        ) -> BasePropertiesBuilder<Id, Ts, Name, Val>,
        Ts: Requirement<timestamp::Timestamp>,
        Name: Requirement<String>,
    {
        FeatureBuilder {
            coordinates: self.coordinates,
            properties: builder_fn(BasePropertiesBuilder::new()).build(),
        }
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Gives a passed in function a mutable reference to the current properties.
    pub fn with_properties_mut<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut P),
    {
        func(&mut self.properties);
        self
    }

    /// Gives a passed in function a mutable reference to the current properties, with a fallible
    /// result. If the function returns an [`Err`], that same [`Err`] is returned.
    pub fn try_with_properties_mut<F, E>(mut self, func: F) -> Result<Self, E>
    where
        F: FnOnce(&mut P) -> Result<(), E>,
    {
        func(&mut self.properties)?;
        Ok(self)
    }

    /// Gives a passed in function a mutable reference to the current coordinates.
    pub fn with_coordinates_mut<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut C),
    {
        func(&mut self.coordinates);
        self
    }

    /// Gives a passed in function a mutable reference to the current coordinates, with a fallible
    /// result. If the function returns an [`Err`], that same [`Err`] is returned.
    pub fn try_with_coordinates_mut<F, E>(mut self, func: F) -> Result<Self, E>
    where
        F: FnOnce(&mut C) -> Result<(), E>,
    {
        func(&mut self.coordinates)?;
        Ok(self)
    }
}

impl<C, P> FeatureBuilder<C, P> {
    /// Inserts a new property map.
    pub fn new_properties<P2>(self, args: P2::RequiredArgs) -> FeatureBuilder<C, P2>
    where
        P2: Properties,
    {
        FeatureBuilder {
            coordinates: self.coordinates,
            properties: P2::new(args),
        }
    }
}

impl<C, P> FeatureBuilder<C, P>
where
    C: Into<AnyCoordinate>,
{
    /// Converts the inner coordinate into [`AnyCoordinate`].
    #[allow(clippy::wrong_self_convention)]
    pub fn as_any_coordinate(self) -> FeatureBuilder<AnyCoordinate, P> {
        FeatureBuilder {
            coordinates: self.coordinates.into(),
            properties: self.properties,
        }
    }
}

impl Default for FeatureBuilder<(), ()> {
    fn default() -> Self {
        FeatureBuilder::empty()
    }
}

impl<P> FeatureBuilder<(), P>
where
    P: Default,
{
    /// Assembles a new [`FeatureBuilder`], with default properties.
    pub fn with_default_properties() -> Self {
        Self {
            coordinates: (),
            properties: P::default(),
        }
    }
}

impl<P> FeatureBuilder<(), P>
where
    P: Properties,
{
    /// Assembles a new [`FeatureBuilder`] with empty coordinates, but instantiates the
    /// properties with the arguments required for [`Properties::new`].
    pub fn new_with_args(args: P::RequiredArgs) -> Self {
        Self {
            coordinates: (),
            properties: P::new(args),
        }
    }
}

impl<C, P> FeatureBuilder<C, P>
where
    P: TimedProps,
{
    /// Sets the timestamp on the underlying type that implements [`Properties`].
    pub fn set_timestamp(mut self, timestamp: timestamp::Timestamp) -> Self {
        self.properties.set_timestamp(timestamp);
        self
    }
}

impl<C, P> FeatureBuilder<C, P>
where
    P: Properties,
{
    /// Sets the name
    pub fn set_name<S>(mut self, name: S) -> Self
    where
        S: Into<P::Name>,
    {
        self.properties.set_name(name.into());
        self
    }

    /// Inserts a property into the underlying [`Properties`] map. Takes a non-[`Into`] type, that
    /// way if the compiler cannot figure out a generic, using this once instead of only using
    /// [`insert_property_into`] will tell the compiler the concrete type.
    ///
    /// [`insert_property_into`]: [`Self::insert_property_into`]
    pub fn insert_property<S>(mut self, key: S, value: P::Value) -> Self
    where
        S: Into<String>,
    {
        self.properties.insert(key.into(), value);
        self
    }

    /// Inserts a property into the underlying [`Properties`] map, if it is [`Some`]. Otherwise,
    /// this is a no-op.
    ///
    /// Takes a non-[`Into`] type, that way if the compiler cannot figure out a generic, using
    /// this once instead of only using [`insert_property_into`] will tell the compiler the
    /// concrete type.
    ///
    /// [`insert_property_into`]: [`Self::insert_property_into`]
    pub fn insert_property_if_some<S>(self, key: S, value_opt: Option<P::Value>) -> Self
    where
        S: Into<String>,
    {
        match value_opt {
            Some(value) => self.insert_property(key.into(), value),
            _ => self,
        }
    }

    /// Same as [`insert_property`], but takes a generic value that can be converted to
    /// [`PropertyMap::Value`]
    ///
    /// [`insert_property`]: [`Self::insert_property`]
    pub fn insert_property_into<S, V>(self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<P::Value>,
    {
        self.insert_property(key.into(), value.into())
    }

    /// Takes an iterator of key/value pairs, and inserts them into the underlying
    /// [`Properties`] map.
    pub fn extend_properties<I, S, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<P::Value>,
    {
        for (key, item) in iter.into_iter() {
            self.properties.insert(key.into(), item.into());
        }
        self
    }
}

impl<C, P> FeatureBuilder<C, P>
where
    P: Properties + PropertyMap<Value = serde_json::Value>,
{
    /// Inserts a property that is serializable into [`serde_json::Value`].
    pub fn serialize_property<S, V>(
        mut self,
        key: S,
        value: V,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        S: Into<String>,
        V: serde::Serialize,
    {
        match serde_json::to_value(value) {
            Ok(value) => {
                self.properties.insert(key.into(), value);
                Ok(self)
            }
            Err(err) => Err((self, err)),
        }
    }

    /// Serializes many properties from an iterator over key-value pairs.
    pub fn serialize_properties<I, S, V>(
        mut self,
        iter: I,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: serde::Serialize,
    {
        for (key, value) in iter.into_iter() {
            self = self.serialize_property(key, value)?;
        }

        Ok(self)
    }
}

impl<C, P> FeatureBuilder<C, P>
where
    P: DisplayProps,
{
    /// Inserts a display property into the underlying [`DisplayProps`] map.
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    pub fn insert_display_property<S>(
        mut self,
        key: S,
        value: <P::DisplayProps as PropertyMap>::Value,
    ) -> Self
    where
        S: Into<String>,
    {
        self.properties
            .display_props_mut()
            .insert(key.into(), value);
        self
    }

    /// Inserts a property into the underlying [`DisplayProps`] map, if it is [`Some`]. Otherwise,
    /// this is a no-op.
    ///
    /// Takes a non-[`Into`] type, that way if the compiler cannot figure out a generic, using
    /// this once instead of only using [`insert_property_into`] will tell the compiler the
    /// concrete type.
    ///
    /// [`insert_property_into`]: [`Self::insert_property_into`]
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    pub fn insert_display_property_if_some<S>(
        self,
        key: S,
        value_opt: Option<<P::DisplayProps as PropertyMap>::Value>,
    ) -> Self
    where
        S: Into<String>,
    {
        match value_opt {
            Some(value) => self.insert_display_property(key.into(), value),
            _ => self,
        }
    }

    /// Inserts a display property into the underlying [`DisplayProps`] map.
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    pub fn insert_display_property_into<S, V>(self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<<P::DisplayProps as PropertyMap>::Value>,
    {
        self.insert_display_property(key.into(), value.into())
    }

    /// Takes an iterator of key/value pairs, and inserts them into the underlying
    /// [`DisplayProps`] map.
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    pub fn extend_display_properties<I, S, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<<P::DisplayProps as PropertyMap>::Value>,
    {
        let display_props = self.properties.display_props_mut();

        for (key, item) in iter.into_iter() {
            display_props.insert(key.into(), item.into());
        }

        self
    }
}

impl<C, P> FeatureBuilder<C, P>
where
    P: DisplayProps,
    <P as DisplayProps>::DisplayProps: PropertyMap<Value = serde_json::Value>,
{
    /// Inserts a display property that is serializable into [`serde_json::Value`].
    pub fn serialize_display_property<S, V>(
        mut self,
        key: S,
        value: V,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        S: Into<String>,
        V: serde::Serialize,
    {
        match serde_json::to_value(value) {
            Ok(value) => {
                self.properties
                    .display_props_mut()
                    .insert(key.into(), value);
                Ok(self)
            }
            Err(err) => Err((self, err)),
        }
    }

    /// Serializes many display properties from an iterator over key-value pairs.
    pub fn serialize_display_properties_properties<I, S, V>(
        mut self,
        iter: I,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: serde::Serialize,
    {
        for (key, value) in iter.into_iter() {
            self = self.serialize_display_property(key, value)?;
        }

        Ok(self)
    }
}
