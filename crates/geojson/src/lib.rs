#![feature(
    associated_type_defaults,
    vec_into_raw_parts,
    unboxed_closures,
    fn_traits,
    associated_const_equality,
    array_try_from_fn
)]
#[warn(missing_docs, missing_debug_implementations)]
pub mod builder;
pub mod collection;
pub mod feature;
pub mod geojson;
pub mod geometry;
mod macros;
pub mod properties;
mod util;

// #[cfg(feature = "axum")]
// mod axum_impls;

pub use geojson_derive::GeoJsonProps;

pub use self::collection::FeatureCollection;
pub use self::feature::Feature;
pub use self::geojson::GeoJson;
pub use self::geometry::Coordinate;
pub use self::properties::{Properties, PropertyMap};

/// Common type aliases for well-known [`Feature`]s.
pub mod common {
    // use super::*;
    // use properties::cmd_center_props;

    /*
    /// Alias to [`Feature<geometry::Point, cmd_center_props::SightingProps>`]. Still generic over
    /// the inner [`PropertyMap::Value`].
    pub type SightingFeature<Val>
        = Feature<geometry::Point, cmd_center_props::SightingProps<Val>>;

    /// Alias to [`FeatureCollection<geometry::Point, cmd_center_props::SightingProps>`]. Still
    /// generic over the inner [`PropertyMap::Value`].
    pub type SightingFeatureCollection<Val>
        = FeatureCollection<geometry::Point, cmd_center_props::SightingProps<Val>>;


    /// Alias to [`Feature<geometry::Polygon, cmd_center_props::CmdCenterProps>`]. Still generic
    /// over the inner [`PropertyMap::Value`].
    pub type PolygonFeature<Val>
        = Feature<geometry::Polygon, cmd_center_props::CmdCenterProps<Val>>;

    /// Alias to [`FeatureCollection<geometry::Polygon, cmd_center_props::CmdCenterProps>`]. Still
    /// generic over the inner [`PropertyMap::Value`].
    pub type PolygonFeatureCollection<Val>
        = FeatureCollection<geometry::Polygon, cmd_center_props::CmdCenterProps<Val>>;

    /// Alias to [`Feature<geometry::AnyCoordinate, cmd_center_props::CmdCenterProps>`]. Still
    /// generic over the inner [`PropertyMap::Value`].
    pub type GenericFeature<Val>
        = Feature<geometry::AnyCoordinate, cmd_center_props::CmdCenterProps<Val>>;

    /// Alias to [`FeatureCollection<geometry::AnyCoordinate, cmd_center_props::CmdCenterProps>`].
    /// Still generic over the inner [`PropertyMap::Value`].
    pub type GenericFeatureCollection<Val>
        = FeatureCollection<geometry::AnyCoordinate, cmd_center_props::CmdCenterProps<Val>>;
    */
}

/// Private trait for setting bounds on publicly exposed but privately implemented traits.
mod private {
    pub trait Sealed {}

    impl Sealed for timestamp::Timestamp {}

    impl Sealed for String {}

    impl<T> Sealed for Option<T> where T: Sealed {}
}

/// A sealed trait that helps get at the properties/coordinate types for [`Feature`],
/// [`FeatureCollection`] and [`GeoJson`] types.
///
/// For example, instead of defining generics, this trait can allow types to reference
/// coordinate and property types without needing to explicitely know it ahead of time.
///
/// ```
/// use std::marker::PhantomData;
///
/// use geojson::geometry::Point;
/// use geojson::{Feature, FeatureCollection, GeoJsonType};
///
/// type PointFeature = Feature<Point, ()>;
/// type PointFeatureCollection = FeatureCollection<Point, ()>;
///
/// #[derive(Debug, PartialEq)]
/// struct TypeChecker<T> {
///     marker: PhantomData<T>,
/// }
///
/// impl<T> TypeChecker<T> {
///     fn new() -> Self {
///         Self {
///             marker: PhantomData,
///         }
///     }
/// }
///
/// let point_feat = TypeChecker::<<PointFeature as GeoJsonType>::Coordinate>::new();
/// let point_collec = TypeChecker::<<PointFeatureCollection as GeoJsonType>::Coordinate>::new();
///
/// assert_eq!(point_feat, point_collec);
/// ```
/// But, where-as that works, this will fail since the coordinate types differ:
///
/// ```compile_fail
/// # use std::marker::PhantomData;
/// # use geojson::{
/// #     GeoJsonType,
/// #     Feature,
/// #     FeatureCollection,
/// #     geometry::Point,
/// # };
/// use geojson::geometry::LineString;
/// # type PointFeatureCollection = FeatureCollection<Point, ()>;
/// type LineStringFeature = Feature<LineString, ()>;
/// # #[derive(Debug, PartialEq)]
/// # struct TypeChecker<T> {
/// #    marker: PhantomData<T>
/// # }
/// # impl<T> TypeChecker<T> {
/// #     fn new() -> Self {
/// #         Self { marker: PhantomData }
/// #     }
/// # }
///
/// let line_string_feat = TypeChecker::<<LineStringFeature as GeoJsonType>::Coordinate>::new();
/// # let point_collec = TypeChecker::<<PointFeatureCollection as GeoJsonType>::Coordinate>::new();
/// assert_eq!(line_string_feat, point_collec);
/// ```
///
/// Once `inherent_associated_types` is stabilized, this trait should be replaced with associated
/// types on the [`Feature`], [`FeatureCollection`] and [`GeoJson`] types themselves.
pub trait GeoJsonType: private::Sealed {
    /// The coordinate type used by [`Self`]
    type Coordinate;
    /// the property type used by [`Self`].
    type Properties;
}

macro_rules! impl_geojson_type {
    ($($type:ident),* $(,)?) => {
        $(
            impl<C, P> private::Sealed for $type<C, P> {}

            impl<C, P> GeoJsonType for $type<C, P> {
                #[doc = "The coordinate type used by [`"]
                #[doc = stringify!($type)]
                #[doc = "`]."]
                type Coordinate = C;

                #[doc = "The property type used by [`"]
                #[doc = stringify!($type)]
                #[doc = "`]."]
                type Properties = P;
            }
        )*
    };
}

impl_geojson_type!(Feature, FeatureCollection, GeoJson);

#[cfg(test)]
mod tests {
    use geometry::Point;
    use properties::base_props::BaseProperties;
    use serde_json::json;
    use timestamp::Timestamp;

    use super::*;

    #[test]
    fn test_sighting_builder() {
        let id = uuid::Uuid::new_v4();

        let feature: Feature<Point, BaseProperties> = Feature::builder()
            .point_try_from((-123.456, 45.0))
            .expect("invalid coordinates")
            .with_base_prop_builder(|builder| {
                builder
                    .set_id(id)
                    .with_required_timestamp(Timestamp::UNIX_EPOCH)
                    .insert_property("client", "test-client")
                    .insert_property("dataType", "sighting")
                    .set_required_name("test-name")
            })
            .insert_property("test-property", serde_json::Value::from(5u32))
            .insert_property_into("test-property-2", "prop-value")
            .insert_display_property("title", serde_json::Value::from("test-title"))
            .build();

        let serialized_value = serde_json::to_value(&feature).expect("could not serialize feature");

        let expected_value = json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [-123.456f64, 45.0f64]
            },
            "properties": {
                "client": "test-client",
                "epoch": 0.0f64,
                "dataType": "sighting",
                "name": "test-name",
                "id": id,
                "displayProperties": {
                    "title": "test-title"
                },
                "test-property": 5u32,
                "test-property-2": "prop-value"
            }
        });

        assert_eq!(serialized_value, expected_value);
        let deserialized_result: Result<Feature<Point, BaseProperties>, serde_json::Error> =
            serde_json::from_value(serialized_value.clone());

        if let Err(err) = deserialized_result {
            println!("{:?}", err.to_string());
        }

        let deserialized_value: Feature<_, _> =
            serde_json::from_value(serialized_value).expect("could not deserialize value");

        assert_eq!(deserialized_value, feature);

        let deserialized_expected_value: Feature<_, _> =
            serde_json::from_value(expected_value).expect("could not deserialize expected value");

        assert_eq!(deserialized_expected_value, feature);
    }
}
