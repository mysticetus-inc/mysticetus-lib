use std::sync::Arc;

use super::bindings::Table;
use super::client::InnerClient;
use super::route;
use super::table::{TableClient, TableClientRef};

#[derive(Debug, Clone)]
pub struct DatasetClient<D> {
    dataset_name: D,
    inner: Arc<InnerClient>,
}

impl<D> DatasetClient<D> {
    #[inline]
    pub(super) const fn from_parts(dataset_name: D, inner: Arc<InnerClient>) -> Self {
        Self {
            dataset_name,
            inner,
        }
    }

    pub fn table<T>(&self, table_name: T) -> TableClient<D, T>
    where
        D: Clone,
    {
        TableClient::from_parts(
            self.dataset_name.clone(),
            table_name,
            Arc::clone(&self.inner),
        )
    }

    pub fn table_ref<T>(&self, table_name: T) -> TableClientRef<'_, &D, T> {
        TableClientRef::from_parts(&self.dataset_name, table_name, &self.inner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DatasetClientRef<'a, D> {
    dataset_name: D,
    inner: &'a InnerClient,
}

impl<'a, D> DatasetClientRef<'a, D> {
    #[inline]
    pub(super) const fn from_parts(dataset_name: D, inner: &'a InnerClient) -> Self {
        Self {
            dataset_name,
            inner,
        }
    }

    pub fn table_ref<T>(&self, table_name: T) -> TableClientRef<'_, &D, T> {
        TableClientRef::from_parts(&self.dataset_name, table_name, self.inner)
    }

    pub async fn create_table(&self, table: Table) -> crate::Result<Table>
    where
        D: super::Identifier,
    {
        let url = route!(self.inner; "datasets" self.dataset_name "tables");
        let table: Table = self.inner.post(url, table).await?.json().await?;
        Ok(table)
    }
}
