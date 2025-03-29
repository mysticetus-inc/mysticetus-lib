spanner_rs::row! {
    #[derive(Debug, Clone, PartialEq)]
    #[spanner(table = "SharedSgts")]
    pub struct SharedSightings {
        #[spanner(pk = 1)]
        pub sighting_time: timestamp::Timestamp,
        #[spanner(pk = 2)]
        pub geo_hash: String,
        #[spanner(pk = 3)]
        pub sighting_id: uuid::Uuid,
        pub name: Option<String>,
        pub longitude: f64,
        pub latitude: f64,
        pub properties: Option<String>,
        pub list: Vec<String>,
    }
}
