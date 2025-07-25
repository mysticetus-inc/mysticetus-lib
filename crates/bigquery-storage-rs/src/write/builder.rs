use std::fmt;

use gcp_auth_provider::Scope;
use protos::bigquery_storage::big_query_write_client::BigQueryWriteClient;
use protos::bigquery_storage::write_stream::WriteMode;
use protos::bigquery_storage::{CreateWriteStreamRequest, WriteStream};

use super::default::load_default_schema;
use super::stream_types::{self, WriteStreamType};
use super::{BigQueryStorageClient, Error, WriteSession};

pub struct WriteSessionBuilder<W, D, T, Schema = ()> {
    client: BigQueryStorageClient,
    stream_type: W,
    dataset_id: D,
    table_id: T,
    schema: Schema,
}

impl WriteSessionBuilder<(), (), (), ()> {
    pub fn new(client: BigQueryStorageClient) -> Self {
        Self {
            client,
            stream_type: (),
            dataset_id: (),
            table_id: (),
            schema: (),
        }
    }
}

impl<D, T, Schema> WriteSessionBuilder<(), D, T, Schema> {
    /// Marks the stream as being a [`Buffered`] stream.
    ///
    /// [`Buffered`]: [`stream_types::Buffered`]
    pub fn buffered_stream(self) -> WriteSessionBuilder<stream_types::Buffered, D, T, Schema> {
        WriteSessionBuilder {
            client: self.client,
            stream_type: stream_types::Buffered,
            dataset_id: self.dataset_id,
            table_id: self.table_id,
            schema: self.schema,
        }
    }

    /// Marks the stream as being a [`Pending`] stream.
    ///
    /// [`Pending`]: [`stream_types::Pending`]
    pub fn pending_stream(self) -> WriteSessionBuilder<stream_types::Pending, D, T, Schema> {
        WriteSessionBuilder {
            client: self.client,
            stream_type: stream_types::Pending,
            dataset_id: self.dataset_id,
            table_id: self.table_id,
            schema: self.schema,
        }
    }

    /// Marks the stream as being a [`Committed`] stream.
    ///
    /// [`Committed`]: [`stream_types::Committed`]
    pub fn committed_stream(self) -> WriteSessionBuilder<stream_types::Committed, D, T, Schema> {
        WriteSessionBuilder {
            client: self.client,
            stream_type: stream_types::Committed,
            dataset_id: self.dataset_id,
            table_id: self.table_id,
            schema: self.schema,
        }
    }
}

impl<W, T, Schema> WriteSessionBuilder<W, (), T, Schema> {
    /// The BigQuery dataset ID
    pub fn dataset_id<D>(self, dataset_id: D) -> WriteSessionBuilder<W, D, T, Schema> {
        WriteSessionBuilder {
            client: self.client,
            stream_type: self.stream_type,
            dataset_id,
            table_id: self.table_id,
            schema: self.schema,
        }
    }
}

impl<W, D, Schema> WriteSessionBuilder<W, D, (), Schema> {
    /// The BigQuery Table to initialize a write stream from.
    pub fn table_id<T>(self, table_id: T) -> WriteSessionBuilder<W, D, T, Schema> {
        WriteSessionBuilder {
            client: self.client,
            stream_type: self.stream_type,
            dataset_id: self.dataset_id,
            table_id,
            schema: self.schema,
        }
    }
}

impl<W, D, T> WriteSessionBuilder<W, D, T, ()>
where
    D: fmt::Display,
    T: fmt::Display,
{
    async fn create_inner(&mut self, name: String) -> Result<WriteStream, Error> {
        let table_info = self
            .client
            .build_table_info(&self.dataset_id, &self.table_id);

        let request = CreateWriteStreamRequest {
            parent: table_info.table,
            write_stream: Some(WriteStream {
                location: String::new(),
                name,
                create_time: None,
                table_schema: None,
                commit_time: None,
                r#type: stream_types::Default::to_type() as i32,
                write_mode: WriteMode::Insert as i32,
            }),
        };

        let mut client = BigQueryWriteClient::new(self.client.channel.clone());

        let write_stream = client.create_write_stream(request).await?.into_inner();
        Ok(write_stream)
    }

    /// Gets the default, [`Commit`] based stream.
    pub async fn get_default_stream<R>(
        self,
    ) -> Result<WriteSession<stream_types::Default, R>, Error> {
        // re-use this path for the default stream name later on
        let mut qual_path = format!(
            "projects/{project_id}/datasets/{dataset_id}/tables/{table_id}",
            project_id = self.client.channel.auth().project_id(),
            dataset_id = self.dataset_id,
            table_id = self.table_id,
        );

        // see the [`crate::storage::write::default`] module docs for more info
        let schema = load_default_schema(&qual_path, self.client.channel.auth(), None).await?;

        qual_path.push_str("/streams/_default");

        let write_stream = WriteStream {
            name: qual_path,
            r#type: stream_types::Default::to_type() as i32,
            ..Default::default()
        };

        let schema = super::Schema::from_table_schema(schema)?;

        WriteSession::new_inner(
            write_stream,
            self.client.channel,
            stream_types::Default,
            Some(schema),
        )
    }
}

impl<W, D, T> WriteSessionBuilder<W, D, T>
where
    W: stream_types::WriteStreamType,
    D: fmt::Display,
    T: fmt::Display,
{
    /// Creates the [`WriteSession`] for this stream.
    pub async fn create<R>(mut self) -> Result<WriteSession<W, R>, Error> {
        let write_stream = self.create_inner(String::new()).await?;
        WriteSession::new_inner(write_stream, self.client.channel, self.stream_type, None)
    }
}
