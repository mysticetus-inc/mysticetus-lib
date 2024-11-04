//! Builders for [`Feature`]s and [`FeatureCollection`]s.
//!
//! [`Feature`]: [`crate::Feature`]
//! [`FeatureCollection`]: [`crate::FeatureCollection`]

mod collection;
mod feature;
mod properties;

pub use collection::{FeatureCollectionBuilder, NewFeatureBuilder};
pub use feature::FeatureBuilder;
pub use properties::BasePropertiesBuilder;

/*
#[test]
fn test_builder() {
    use crate::Feature;
    use crate::geometry::LineString;
    use crate::properties::cmd_center_props::SightingProps;

    let feature: Feature<_, SightingProps<serde_json::Value>>
        = FeatureBuilder::new_with_id(uuid::Uuid::new_v4())
        .coordinates(LineString::random_with_len(10))
        .insert_property("test2", "test")
        .set_timestamp(timestamp::Timestamp::now())
        .insert_display_property("wow", "neat")
        .build();

    let geojson_str = serde_json::to_string_pretty(&feature).unwrap();

    println!("{geojson_str}");
}
*/
