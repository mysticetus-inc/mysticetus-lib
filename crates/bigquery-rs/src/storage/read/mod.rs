use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use gcp_auth_channel::Scope;
use http::HeaderValue;

pub mod avro_de;

use apache_avro::Schema;
use protos::bigquery_storage::big_query_read_client::BigQueryReadClient;
use protos::bigquery_storage::read_session::{self, TableReadOptions};
use protos::bigquery_storage::{self, CreateReadSessionRequest, DataFormat};
use serde::de::{Deserialize, DeserializeSeed};

use super::{BigQueryStorageClient, Error};

mod session;
mod stream;

pub use session::ReadSession;
pub use stream::ReadStream;

#[derive(Debug, Clone)]
pub struct ReadClient(BigQueryStorageClient);

impl From<BigQueryStorageClient> for ReadClient {
    fn from(inner: BigQueryStorageClient) -> Self {
        Self(inner)
    }
}

impl ReadClient {
    /// Builds a read client, internally building a [`BigQueryStorageClient`].
    pub async fn new(project_id: &'static str, scope: Scope) -> Result<Self, Error> {
        BigQueryStorageClient::new(project_id, scope)
            .await
            .map(Self)
    }

    pub fn session_builder(&self) -> ReadSessionBuilder<(), ()> {
        ReadSessionBuilder::new(self.0.clone())
    }

    pub async fn session<D, T, O>(
        &self,
        dataset_id: D,
        table_id: T,
    ) -> Result<ReadSession<PhantomData<O>>, Error>
    where
        D: fmt::Display,
        T: fmt::Display,
        for<'de> O: Deserialize<'de>,
    {
        self.session_builder()
            .table_id(table_id)
            .dataset_id(dataset_id)
            .create()
            .await
    }
}

pub struct ReadSessionBuilder<D, T> {
    client: BigQueryStorageClient,
    dataset_id: D,
    table_id: T,
    trace_id: uuid::Uuid,
    fields: Option<Vec<String>>,
    row_restriction: Option<String>,
    max_stream_count: u16,
}

impl ReadSessionBuilder<(), ()> {
    fn new(client: BigQueryStorageClient) -> Self {
        Self {
            client,
            dataset_id: (),
            table_id: (),
            trace_id: uuid::Uuid::new_v4(),
            fields: None,
            row_restriction: None,
            max_stream_count: 0,
        }
    }
}

impl<D, T> ReadSessionBuilder<D, T> {
    pub fn with_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.fields = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_row_restriction<S>(mut self, row_restriction: S) -> Self
    where
        S: Into<String>,
    {
        self.row_restriction = Some(row_restriction.into());
        self
    }

    pub fn max_stream_count(mut self, max_stream_count: u16) -> Self {
        self.max_stream_count = max_stream_count;
        self
    }
}

impl<D> ReadSessionBuilder<D, ()> {
    pub fn table_id<T>(self, table_id: T) -> ReadSessionBuilder<D, T> {
        ReadSessionBuilder {
            client: self.client,
            dataset_id: self.dataset_id,
            table_id,
            trace_id: self.trace_id,
            fields: self.fields,
            row_restriction: self.row_restriction,
            max_stream_count: self.max_stream_count,
        }
    }
}

impl<T> ReadSessionBuilder<(), T> {
    pub fn dataset_id<D>(self, dataset_id: D) -> ReadSessionBuilder<D, T> {
        ReadSessionBuilder {
            client: self.client,
            dataset_id,
            table_id: self.table_id,
            trace_id: self.trace_id,
            fields: self.fields,
            row_restriction: self.row_restriction,
            max_stream_count: self.max_stream_count,
        }
    }
}

impl<D, T> ReadSessionBuilder<D, T>
where
    D: fmt::Display,
    T: fmt::Display,
{
    pub async fn create<O>(self) -> Result<ReadSession<PhantomData<O>>, Error>
    where
        for<'de> O: Deserialize<'de>,
    {
        self.create_with_seed(PhantomData).await
    }

    pub async fn create_with_seed<S>(self, seed: S) -> Result<ReadSession<S>, Error>
    where
        for<'de> S: DeserializeSeed<'de> + Clone,
    {
        let table_info = self
            .client
            .build_table_info(&self.dataset_id, &self.table_id);

        let header_str = format!("read_session.table={}", table_info.table);
        let table_header = HeaderValue::from_str(&header_str)?;

        let read_options = match (self.fields, self.row_restriction) {
            (None, None) => None,
            (fields, row_restriction) => Some(TableReadOptions {
                sample_percentage: None,
                response_compression_codec: None,
                selected_fields: fields.unwrap_or_default(),
                row_restriction: row_restriction.unwrap_or_default(),
                output_format_serialization_options: None,
            }),
        };

        let create_request = CreateReadSessionRequest {
            parent: table_info.parent,
            preferred_min_stream_count: num_cpus::get() as i32,
            max_stream_count: self.max_stream_count.into(),
            read_session: Some(bigquery_storage::ReadSession {
                data_format: DataFormat::Avro as i32,
                table: table_info.table,
                read_options,
                trace_id: self.trace_id.to_string(),
                table_modifiers: None,
                // all other fields are output only
                ..Default::default()
            }),
        };

        let mut channel = self
            .client
            .channel
            .clone()
            .attach_header()
            .static_key(super::GOOG_REQ_PARAMS_KEY)
            .value(table_header)
            .with_scope(Scope::BigQueryReadOnly);

        let mut client = BigQueryReadClient::new(&mut channel);

        let mut read_session = client
            .create_read_session(create_request)
            .await?
            .into_inner();

        tracing::info!(
            message = "created read stream session",
            name = read_session.name.as_str(),
            table = read_session.table.as_str(),
            trace = read_session.trace_id.as_str(),
            stream_count = read_session.streams.len(),
        );

        let schema = match read_session.schema.take() {
            Some(read_session::Schema::AvroSchema(avro_schema)) => avro_schema,
            Some(read_session::Schema::ArrowSchema(_)) => return Err(Error::ArrowNotSupported),
            None => {
                return Err(Error::Status(tonic::Status::internal(
                    "no schema returned from create read session",
                )));
            }
        };

        let schema = Schema::parse_str(&schema.schema)?;

        Ok(ReadSession {
            read_session,
            schema: Arc::new(schema),
            client: self.client,
            seed,
        })
    }
}
