#![feature(
    trait_alias,
    slice_as_chunks,
    type_changing_struct_update,
    box_into_inner,
    seek_stream_len,
    const_trait_impl,
    let_chains,
    const_swap
)]

#[macro_use]
extern crate const_format;

#[macro_use]
extern crate tracing;

#[cfg(any(feature = "storage-read", feature = "storage-write"))]
pub mod storage;
// pub use storage::{BigQueryReadSession, BigQueryStorageClient};

mod error;
#[cfg(feature = "rest")]
pub mod rest;
pub use error::Error;

const PROJECT_ID: &str = "mysticetus-oncloud";

const DATASET_ID: &str = "oncloud_local_mrudisel_arch";

const PARENT_PATH: &str = formatcp!("projects/{}", PROJECT_ID);

use serde::{Deserialize, Serialize};

/// Type alias to [`core::result::Result<T, Error>`].
pub type Result<T> = core::result::Result<T, Error>;

mod private {
    /// Sealed trait for use throughout the crate
    pub trait Sealed {}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrackMark {
    #[serde(flatten)]
    pub point: Point,

    pub timestamp: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeoTrack {
    guid: String,
    #[serde(rename = "type")]
    geo_type: String,
    group_id: u64,
    pk: u64,
    vessel: Option<String>,

    #[serde(deserialize_with = "serde_json::Value::deserialize")]
    blob: serde_json::Value,

    station_id: Option<String>,
    name: Option<String>,
}

/*
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn try_create_session() -> Result<(), Box<dyn std::error::Error>> {
    // Match the number of threads available on a cloud run instance
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()?;

    let client = BigQueryStorageClient::init(PROJECT_ID).await?;

    let mut session = client.session_builder()
        .dataset_id(DATASET_ID)
        //.table("geotracks")
        //.fields(vec!["guid", "type", "group_id", "pk", "vessel", "blob", "station_id", "name"])
        //.filter("type = 'Polygon' AND vessel LIKE 'OCS%'")
        .table("trackmarks")
        .fields(vec!["lat", "lon", "timestamp"])
        .filter("timestamp > 0 AND vessel = 'Track Stri'")
        .max_stream_count(10u16)
        .create()
        .await?;

    println!("created session");
    let start = std::time::Instant::now();

    let rows: Vec<TrackMark> = session.read_rows().await?;

    let elapsed = start.elapsed();

    let n_rows = rows.len();

    println!("recieved {} rows total", n_rows);
    println!("took {} ms", elapsed.as_millis());
    println!("{} track marks per second", n_rows as f64 / (elapsed.as_millis() * 1000) as f64);

    println!("{:#?}", rows[0]);

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
*/
