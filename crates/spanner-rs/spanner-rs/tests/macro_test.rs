spanner_rs::table! {
    #[derive(Debug, Clone, PartialEq)]
    #[spanner(table_name = "SharedSgts")]
    pub struct SharedSightings {
        #[spanner(pk = 1)]
        sighting_time: timestamp::Timestamp,
        #[spanner(pk = 2)]
        geo_hash: String,
        #[spanner(pk = 3)]
        sighting_id: uuid::Uuid,

        name: Option<String>,
        longitude: f64,
        latitude: f64,
        properties: Option<String>,
        list: Vec<String>,
    }
}
