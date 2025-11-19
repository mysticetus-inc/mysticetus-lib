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
    use geo::geom::{Line, Polygon};
    use geometry::Point;
    use properties::base_props::BaseProperties;
    use serde_json::json;
    use timestamp::Timestamp;
    use uuid::Uuid;

    use super::*;
    use crate::properties::cmd_center_props::LeaseArea;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
    pub struct LeaseAreaProps<'a> {
        id: Uuid,
        data_type: LeaseArea,
        name: &'a str,
        parent: &'a str,
        radius_meters: usize,
        sort_key: i32,
    }

    fn generate_ring<'a>(
        center: Point,
        radius_meters: usize,
        sort_key: i32,
        parent: &'a str,
        name: &'a str,
        points: usize,
    ) -> Feature<Polygon, LeaseAreaProps<'a>> {
        let center = geo::NormalVec::from_point(center);

        let mut line = Line::with_capacity(points + 1);

        for i in 0..points {
            let bearing_rad = 2.0 * std::f64::consts::PI * ((i as f64) / (points as f64));
            let pt = center.extend_point(bearing_rad, radius_meters as f64);
            let point = pt.try_into_point().unwrap();
            line.push(point);
        }

        line.push(line.as_slice()[0]);

        let poly = Polygon::from(line);

        Feature::from_coords_and_properties(
            poly,
            LeaseAreaProps {
                id: Uuid::new_v5(
                    &uuid::Uuid::NAMESPACE_DNS,
                    format!("{radius_meters}-{parent}-{name}").as_bytes(),
                ),
                data_type: LeaseArea,
                name,
                sort_key,
                parent,
                radius_meters,
            },
        )
    }

    #[test]
    fn generate_geojson() {
        const ESSINGTON_1: (Point, &str) =
            (Point::new_raw(142.8120570, -39.0957041), "Essington-1");

        const CHARLEMONT_1: (Point, &str) =
            (Point::new_raw(142.6080531, -39.0142688), "Charlemont-1");

        const RINGS: [(usize, i32, &str); 3] = [
            (2500, 3, "Whale Action Zone - 2.5km"),
            (3000, 2, "Glider Exclusion Zone - 3km"),
            (13000, 1, "Whale Action Zone - 13km"),
        ];

        // 2.5km - hsl(0, 96%, 44%) - 0.24
        // 3km - hsl(301, 89%, 46%) - 0.24
        // 13km - hsl(0, 30%, 96%) - 0.24

        for (center, parent) in [ESSINGTON_1, CHARLEMONT_1] {
            let mut collection = FeatureCollection::with_capacity(RINGS.len());

            for (radius, sort_key, name) in RINGS {
                let feature = generate_ring(center, radius, sort_key, parent, name, 256);
                collection.push(feature);
            }

            let json = serde_json::to_string_pretty(&collection).unwrap();

            let dst = format!("{parent}-collection.geojson");
            std::fs::write(&dst, json.as_bytes()).unwrap();
        }
    }

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
