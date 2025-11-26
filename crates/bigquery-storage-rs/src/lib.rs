#![feature(seek_stream_len)]
use std::fmt;
use std::time::Duration;

mod error;
pub(crate) mod proto;
#[cfg(feature = "read")]
pub mod read;
#[cfg(feature = "write")]
pub mod write;

pub type Result<T> = core::result::Result<T, Error>;

pub use error::Error;
use gcp_auth_provider::service::AuthSvc;
use gcp_auth_provider::{Auth, Scopes};
use tonic::transport::{Channel, ClientTlsConfig};

const BQ_HOST: &str = "https://bigquerystorage.googleapis.com";
const BQ_DOMAIN: &str = "bigquerystorage.googleapis.com";

#[cfg(any(feature = "write", feature = "read"))]
const GOOG_REQ_PARAMS_KEY: http::HeaderName =
    http::HeaderName::from_static("x-goog-request-params");

const KEEPALIVE_DURATION: Duration = Duration::from_secs(120);

/// A read/write-agnostic client to the BigQuery Storage API.
#[derive(Debug, Clone)]
pub struct BigQueryStorageClient {
    channel: AuthSvc<Channel>,
}

async fn build_channel() -> Result<Channel> {
    let tls_config = {
        #[cfg(not(feature = "tls-webpki-roots"))]
        {
            ClientTlsConfig::new().domain_name(BQ_DOMAIN)
        }
        #[cfg(feature = "tls-webpki-roots")]
        {
            ClientTlsConfig::new()
                .domain_name(BQ_DOMAIN)
                .with_webpki_roots()
        }
    };

    Channel::from_static(BQ_HOST)
        .user_agent("bigquery-rs")?
        .tcp_keepalive(Some(KEEPALIVE_DURATION))
        .tls_config(tls_config)?
        .connect()
        .await
        .map_err(Error::from)
}

impl BigQueryStorageClient {
    /// Builds a new BQ Storage client.
    pub async fn new(scopes: impl Into<Scopes>) -> Result<Self> {
        let channel = Auth::builder()
            .add_scopes(scopes)
            .channel_with_defaults(BQ_HOST, BQ_DOMAIN)
            .auth(Auth::new_detect())
            .build()
            .await?;

        Ok(Self { channel })
    }

    /// Builds a new BQ Storage client, from an existing auth manager.
    pub async fn from_auth(auth: Auth) -> Result<Self> {
        let channel = auth
            .build_channel()
            .channel_with_defaults(BQ_HOST, BQ_DOMAIN)
            .build()
            .await?;

        Ok(Self { channel })
    }

    /// Gets a handle to a [`ReadClient`].
    ///
    /// [`ReadClient`]: [`read::ReadClient`]
    #[cfg(feature = "read")]
    pub fn read_client(&self) -> read::ReadClient {
        read::ReadClient::from(self.clone())
    }

    /// Converts into a [`ReadClient`].
    ///
    /// If everything is built in chain of builder functions, this prevents
    /// errors from not storing the [`BigQueryStorageClient`] into its own variable.
    ///
    /// [`ReadClient`]: [`read::ReadClient`]
    #[cfg(feature = "read")]
    pub fn into_read_client(self) -> read::ReadClient {
        read::ReadClient::from(self)
    }

    /// Gets a handle to a [`WriteClient`].
    ///
    /// [`WriteClient`]: [`write::WriteClient`]
    #[cfg(feature = "write")]
    pub fn write_client(&self) -> write::WriteClient {
        write::WriteClient::from(self.clone())
    }

    /// Converts into a [`WriteClient`].
    ///
    /// If everything is built in chain of builder functions, this prevents
    /// errors from not storing the [`BigQueryStorageClient`] into its own variable.
    ///
    /// [`WriteClient`]: [`write::WriteClient`]
    #[cfg(feature = "write")]
    pub fn into_write_client(self) -> write::WriteClient {
        write::WriteClient::from(self)
    }

    fn build_table_info<D, T>(&self, dataset_id: &D, table_id: &T) -> TableInfo
    where
        D: fmt::Display,
        T: fmt::Display,
    {
        TableInfo::new(
            self.channel.auth().project_id().as_str(),
            dataset_id,
            table_id,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableInfo {
    pub parent: String,
    pub table: String,
}

impl TableInfo {
    fn new<D, T>(project_id: &str, dataset_id: &D, table_id: &T) -> Self
    where
        D: fmt::Display,
        T: fmt::Display,
    {
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

#[cfg(feature = "read")]
#[ignore = "longrunning test"]
#[tokio::test(flavor = "multi_thread")]
async fn test_read() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let client = BigQueryStorageClient::new(gcp_auth_provider::Scope::BigQueryReadOnly).await?;

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
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
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

#[cfg(test)]
impl TestTrackMark {
    pub fn generate<S, R>(vessel: S, rng: &mut R) -> Self
    where
        R: rand::Rng,
        S: Into<String>,
    {
        use rand::prelude::IndexedRandom;

        const TAGS: &[&str] = &["tag 0", "tag 1", "tag 2", "tag 3", "tag 4", "tag 5"];

        let n_tags = rng.random_range(1..=TAGS.len());
        let mut tags = Vec::with_capacity(n_tags);
        tags.extend(
            TAGS.choose_multiple(rng, n_tags)
                .copied()
                .map(ToOwned::to_owned),
        );

        Self {
            timestamp: TsAsMs(timestamp::Timestamp::now()),
            lon: 360.0 * rng.random::<f64>() - 180.0,
            lat: 180.0 * rng.random::<f64>() - 90.0,
            alt: rng
                .random::<bool>()
                .then(|| 10.0 * rng.random::<f64>() - 5.0),
            vessel: vessel.into(),
            fixed: rng.random_bool(0.9),
            tags,
        }
    }
}

#[cfg(feature = "write")]
#[tokio::test(flavor = "multi_thread")]
async fn test_write() -> Result<()> {
    tracing_subscriber::fmt().init();

    let client = BigQueryStorageClient::new(gcp_auth_provider::Scope::BigQueryReadOnly).await?;

    let write_session = client
        .into_write_client()
        .session_builder()
        .dataset_id("bq_storage_write_test")
        .table_id("test_tracks")
        .get_default_stream::<TestTrackMark>()
        .await?;

    println!("{:#?}", write_session.schema());

    let mut rng = rand::rng();

    let tracks = (0..10)
        .map(|_| TestTrackMark::generate("Test A", &mut rng))
        .collect::<Vec<_>>();

    let resp = write_session.append_rows(tracks).await?;
    println!("{resp:?}");

    Ok(())
}
