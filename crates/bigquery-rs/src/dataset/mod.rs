use bigquery_resources_rs::DatasetReference;
use bigquery_resources_rs::query::{QueryRequest, QueryString};
use shared::arc_or_ref::ArcOrRef;

use super::client::InnerClient;
use crate::BigQueryClient;
use crate::query::QueryBuilder;
use crate::resources::table::Table;
use crate::table::TableClient;

mod table_stream;

pub use table_stream::TableStream;

#[derive(Debug, Clone)]
pub struct DatasetClient<'a, D> {
    dataset_name: D,
    client: ArcOrRef<'a, InnerClient>,
}

impl<'a, D> DatasetClient<'a, D> {
    #[inline]
    pub(super) fn from_parts(
        dataset_name: D,
        client: impl Into<ArcOrRef<'a, InnerClient>>,
    ) -> Self {
        Self {
            dataset_name,
            client: client.into(),
        }
    }

    pub fn query<S>(&self, query: QueryString) -> QueryBuilder<S>
    where
        for<'d> S: From<&'static str> + From<D>,
        D: Clone,
    {
        let mut request = QueryRequest::new(query);

        request.default_dataset = Some(DatasetReference {
            project_id: S::from(self.client.project_id()),
            dataset_id: S::from(self.dataset_name.clone()),
        });

        QueryBuilder::new(
            BigQueryClient {
                inner: self.client.clone().into_arc(),
            },
            request,
        )
    }

    #[inline]
    pub fn into_owned(self) -> DatasetClient<'static, D> {
        DatasetClient {
            dataset_name: self.dataset_name,
            client: self.client.into_owned(),
        }
    }

    #[inline]
    pub fn client(&self) -> BigQueryClient {
        BigQueryClient {
            inner: self.client.as_arc_ref().clone(),
        }
    }

    #[inline]
    pub fn into_client(self) -> BigQueryClient {
        BigQueryClient {
            inner: self.client.into_arc(),
        }
    }

    #[inline]
    pub fn table<T>(&self, table: T) -> TableClient<'_, &D, T> {
        TableClient::from_parts(
            &self.dataset_name,
            table,
            ArcOrRef::Ref(self.client.as_arc_ref()),
        )
    }

    #[inline]
    pub fn into_table<T>(self, table: T) -> TableClient<'a, D, T> {
        TableClient::from_parts(self.dataset_name, table, self.client)
    }
}

impl<'a, D: AsRef<str>> DatasetClient<'a, D> {
    pub fn list_tables(&self, max_page_size: usize) -> TableStream<'_, 'a, D> {
        TableStream::new(self, max_page_size)
    }

    pub async fn create_table(&self, table: Table) -> crate::Result<Table>
    where
        D: AsRef<str>,
    {
        let url = crate::util::append_to_path(self.client.base_url(), &[
            "datasets",
            self.dataset_name.as_ref(),
            "tables",
        ]);

        let resp = self.client.post(url, table).await?;
        super::client::deserialize_json(resp).await
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_table_list() -> crate::Result<()> {
        let client = super::super::BigQueryClient::new(
            "mysticetus-boem",
            gcp_auth_channel::Scope::BigQueryReadOnly,
        )
        .await?;

        let dataset_client = client.dataset("main");

        let tables = dataset_client.list_tables(10).collect().await?;
        println!("{tables:#?}");

        let serialized_tables = serde_json::to_string_pretty(&tables).unwrap();
        tokio::fs::write("tables.json", serialized_tables.as_bytes())
            .await
            .unwrap();
        Ok(())
    }
}
