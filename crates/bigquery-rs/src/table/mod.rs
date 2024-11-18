use shared::arc_or_ref::ArcOrRef;

use super::client::InnerClient;
use crate::BigQueryClient;
use crate::resources::table::Table;
use crate::resources::table_data::TableDataInsertAllResponse;

mod insert_rows;
pub use insert_rows::InsertRowOptions;

#[derive(Debug, Clone)]
pub struct TableClient<'a, D, T> {
    dataset_name: D,
    table_name: T,
    client: ArcOrRef<'a, InnerClient>,
}

impl<'a, D, T> TableClient<'a, D, T> {
    #[inline]
    pub(super) fn from_parts(
        dataset_name: D,
        table_name: T,
        client: impl Into<ArcOrRef<'a, InnerClient>>,
    ) -> Self {
        Self {
            dataset_name,
            table_name,
            client: client.into(),
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
    pub fn into_owned(self) -> TableClient<'static, D, T> {
        TableClient {
            dataset_name: self.dataset_name,
            table_name: self.table_name,
            client: self.client.into_owned(),
        }
    }

    pub async fn insert_rows<A>(&self, rows: A) -> crate::Result<TableDataInsertAllResponse>
    where
        A: IntoIterator,
        A::Item: serde::Serialize,
        D: AsRef<str>,
        T: AsRef<str>,
    {
        self.insert_rows_opt(rows, InsertRowOptions::default())
            .await
    }

    pub async fn insert_rows_opt<A>(
        &self,
        rows: A,
        options: InsertRowOptions,
    ) -> crate::Result<TableDataInsertAllResponse>
    where
        A: IntoIterator,
        A::Item: serde::Serialize,
        D: AsRef<str>,
        T: AsRef<str>,
    {
        let url = crate::util::append_to_path(self.client.base_url(), &[
            "datasets",
            self.dataset_name.as_ref(),
            "tables",
            self.table_name.as_ref(),
            "insertAll",
        ]);

        let payload = insert_rows::InsertRows::new(options, rows);

        let resp = self.client.post(url, payload).await?;

        crate::client::deserialize_json(resp).await
    }

    pub async fn get(&self) -> crate::Result<Table>
    where
        D: AsRef<str>,
        T: AsRef<str>,
    {
        let url = crate::util::append_to_path(self.client.base_url(), &[
            "datasets",
            self.dataset_name.as_ref(),
            "tables",
            self.table_name.as_ref(),
        ]);

        let resp = self.client.get(url).await?;
        super::client::deserialize_json(resp).await
    }

    pub async fn delete(&self) -> crate::Result<()>
    where
        D: AsRef<str>,
        T: AsRef<str>,
    {
        let url = crate::util::append_to_path(self.client.base_url(), &[
            "datasets",
            self.dataset_name.as_ref(),
            "tables",
            self.table_name.as_ref(),
        ]);

        self.client.delete(url).await?;
        Ok(())
    }
}
