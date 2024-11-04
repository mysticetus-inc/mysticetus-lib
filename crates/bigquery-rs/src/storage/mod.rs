use std::fmt;
use std::sync::Arc;
use std::time::Duration;

#[cfg(any(feature = "storage-read", feature = "storage-write"))]
pub(super) mod proto;
#[cfg(feature = "storage-read")]
pub mod read;
#[cfg(feature = "storage-write")]
pub mod write;

use gcp_auth_channel::{AuthChannel, AuthManager};
use tonic::transport::{Channel, ClientTlsConfig};

use crate::Error;

const BQ_HOST: &str = "https://bigquerystorage.googleapis.com";
const BQ_DOMAIN: &str = "bigquerystorage.googleapis.com";

const BQ_SCOPES: &[&str] = &["https://www.googleapis.com/auth/bigquery"];

const GOOG_REQ_PARAMS_KEY: &str = "x-goog-request-params";

const KEEPALIVE_DURATION: Duration = Duration::from_secs(120);

/// A read/write-agnostic client to the BigQuery Storage API.
#[derive(Debug, Clone)]
pub struct BigQueryStorageClient {
    channel: AuthChannel,
    project_id: Arc<String>,
}

async fn build_channel() -> Result<Channel, Error> {
    Channel::from_static(BQ_HOST)
        .user_agent("bigquery-rs")?
        .concurrency_limit(5000)
        .tcp_keepalive(Some(KEEPALIVE_DURATION))
        .tls_config(ClientTlsConfig::new().domain_name(BQ_DOMAIN))?
        .connect()
        .await
        .map_err(Error::from)
}

async fn build_auth_manager() -> Result<AuthManager, Error> {
    AuthManager::new_shared().await.map_err(Error::from)
}

impl BigQueryStorageClient {
    /// Builds a new BQ Storage client.
    pub async fn new<P>(project_id: P) -> Result<Self, Error>
    where
        P: Into<String>,
    {
        let (inner_channel, auth_manager) =
            tokio::try_join!(build_channel(), build_auth_manager(),)?;

        let channel = AuthChannel::builder()
            .with_channel(inner_channel)
            .with_auth_manager(auth_manager)
            .build();

        Ok(Self {
            channel,
            project_id: Arc::new(project_id.into()),
        })
    }

    /// Builds a new BQ Storage client, from an existing auth manager.
    pub async fn from_auth_manager<P, A>(project_id: P, auth_manager: A) -> Result<Self, Error>
    where
        P: Into<String>,
        A: Into<gcp_auth_channel::AuthManager>,
    {
        let channel = build_channel().await?;

        let channel = AuthChannel::builder()
            .with_channel(channel)
            .with_auth_manager(auth_manager.into())
            .build();

        Ok(Self {
            channel,
            project_id: Arc::new(project_id.into()),
        })
    }

    /// Gets a handle to a [`ReadClient`].
    ///
    /// [`ReadClient`]: [`read::ReadClient`]
    #[cfg(feature = "storage-read")]
    pub fn read_client(&self) -> read::ReadClient {
        read::ReadClient::from(self.clone())
    }

    /// Converts into a [`ReadClient`].
    ///
    /// If everything is built in chain of builder functions, this prevents
    /// errors from not storing the [`BigQueryStorageClient`] into its own variable.
    ///
    /// [`ReadClient`]: [`read::ReadClient`]
    #[cfg(feature = "storage-read")]
    pub fn into_read_client(self) -> read::ReadClient {
        read::ReadClient::from(self)
    }

    /// Gets a handle to a [`WriteClient`].
    ///
    /// [`WriteClient`]: [`write::WriteClient`]
    #[cfg(feature = "storage-write")]
    pub fn write_client(&self) -> write::WriteClient {
        write::WriteClient::from(self.clone())
    }

    /// Converts into a [`WriteClient`].
    ///
    /// If everything is built in chain of builder functions, this prevents
    /// errors from not storing the [`BigQueryStorageClient`] into its own variable.
    ///
    /// [`WriteClient`]: [`write::WriteClient`]
    #[cfg(feature = "storage-write")]
    pub fn into_write_client(self) -> write::WriteClient {
        write::WriteClient::from(self)
    }

    fn build_table_info<D, T>(&self, dataset_id: &D, table_id: &T) -> TableInfo
    where
        D: fmt::Display,
        T: fmt::Display,
    {
        TableInfo::new(self.project_id.as_str(), dataset_id, table_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableInfo {
    pub parent: String,
    pub table: String,
}

impl TableInfo {
    fn new<P, D, T>(project_id: P, dataset_id: &D, table_id: &T) -> Self
    where
        P: AsRef<str>,
        D: fmt::Display,
        T: fmt::Display,
    {
        let project_id = project_id.as_ref();

        let mut parent = String::with_capacity(9 + project_id.len());
        parent.push_str("projects/");
        parent.push_str(project_id);

        let mut table = parent.clone();
        table.push_str("/datasets/");

        std::fmt::write(&mut table, format_args!("{}", dataset_id))
            .expect("fmt::write should never fail");

        table.push_str("/tables/");

        std::fmt::write(&mut table, format_args!("{}", table_id))
            .expect("fmt::write should never fail");

        Self { table, parent }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct TrackMark {
    geotrack_id: usize,
    group_id: usize,
    guid: uuid::Uuid,
    vessel: String,
    lon: f64,
    lat: f64,
    alt: Option<f64>,
    gps_fix_status: String,
    timestamp: timestamp::Timestamp,
}

#[ignore = "longrunning test"]
#[tokio::test(flavor = "multi_thread")]
async fn test_read() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let client = BigQueryStorageClient::new("mysticetus-oncloud").await?;

    let read_session = client
        .into_read_client()
        .session_builder()
        .max_stream_count(1)
        .dataset_id("oncloud_production")
        .table_id("trackmarks-no-partition")
        .with_fields([
            "geotrack_id",
            "group_id",
            "guid",
            "vessel",
            "lat",
            "lon",
            "alt",
            "timestamp",
            "gps_fix_status",
        ])
        .with_row_restriction("geotrack_id = 2350")
        .create()
        .await?;

    // partitioned: 17564 rows in 101.676493016
    // not-partitioned: found 17564 rows in 16.770267936 seconds

    let mut tracks: Vec<TrackMark> = Vec::new();

    let start = std::time::Instant::now();
    let mut stream = read_session.stream_all().await?;

    while let Some(mut chunk) = stream.next_batch().await? {
        tracks.append(&mut chunk);
    }

    let elapsed = start.elapsed();

    println!(
        "found {} rows in {} seconds",
        tracks.len(),
        elapsed.as_secs_f64()
    );
    println!("track: {:#?}", tracks.first());
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TsAsMs(timestamp::Timestamp);

impl serde::Serialize for TsAsMs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.as_micros())
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TestTrackMark {
    timestamp: TsAsMs,
    lon: f64,
    lat: f64,
    alt: Option<f64>,
    vessel: String,
    fixed: bool,
    tags: Vec<String>,
}

const TAGS: &[&str] = &["tag 0", "tag 1", "tag 2", "tag 3", "tag 4", "tag 5"];

#[cfg(test)]
impl TestTrackMark {
    pub fn generate<S, R>(vessel: S, rng: &mut R) -> Self
    where
        R: rand::Rng,
        S: Into<String>,
    {
        use rand::seq::SliceRandom;

        let n_tags = rng.gen_range(1..=TAGS.len());
        let mut tags = Vec::with_capacity(n_tags);
        tags.extend(
            TAGS.choose_multiple(rng, n_tags)
                .copied()
                .map(ToOwned::to_owned),
        );

        Self {
            timestamp: TsAsMs(timestamp::Timestamp::now()),
            lon: 360.0 * rng.gen::<f64>() - 180.0,
            lat: 180.0 * rng.gen::<f64>() - 90.0,
            alt: rng.gen::<bool>().then(|| 10.0 * rng.gen::<f64>() - 5.0),
            vessel: vessel.into(),
            fixed: rng.gen_bool(0.9),
            tags,
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_write() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let client = BigQueryStorageClient::new("mysticetus-oncloud").await?;

    let write_session = client
        .into_write_client()
        .session_builder()
        .dataset_id("bq_storage_write_test")
        .table_id("test_tracks")
        .get_default_stream::<TestTrackMark>()
        .await?;

    println!("{:#?}", write_session.get_schemas());

    let mut rng = rand::thread_rng();

    let tracks = (0..10)
        .map(|_| TestTrackMark::generate("Test A", &mut rng))
        .collect::<Vec<_>>();

    let resp = write_session.append_rows(tracks).await?;
    println!("{resp:?}");

    Ok(())
}
