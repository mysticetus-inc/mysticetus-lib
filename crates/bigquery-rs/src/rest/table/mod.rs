use std::cell::RefCell;
use std::sync::Arc;

use super::bindings::TableDataInsertAllResponse;
// re-export bindings in the relevant module
pub use super::bindings::{Table, TableFieldSchema, TableSchema};
use super::client::InnerClient;
use super::{route, Identifier};

pub mod schema_builder;

#[derive(Debug, Clone)]
pub struct TableClient<D, T> {
    dataset_name: D,
    table_name: T,
    inner: Arc<InnerClient>,
}

#[derive(Debug, Clone, Copy)]
pub struct TableClientRef<'a, D, T> {
    dataset_name: D,
    table_name: T,
    inner: &'a InnerClient,
}

impl<D, T> TableClient<D, T> {
    #[inline]
    pub(super) const fn from_parts(
        dataset_name: D,
        table_name: T,
        inner: Arc<InnerClient>,
    ) -> Self {
        Self {
            dataset_name,
            table_name,
            inner,
        }
    }

    pub fn builder(&self) -> schema_builder::TableBuilder<'_, &D, &T> {
        schema_builder::TableBuilder::init(&self.inner, &self.dataset_name, &self.table_name)
    }

    pub async fn insert_rows<A>(&self, rows: A) -> crate::Result<TableDataInsertAllResponse>
    where
        A: IntoIterator,
        A::Item: serde::Serialize,
        D: Identifier,
        T: Identifier,
    {
        let url =
            route!(self.inner; "datasets" self.dataset_name "tables" self.table_name "insertAll");

        let payload = InsertRows {
            skip_invalid_rows: false,
            ignore_unknown_values: false,
            trace_id: uuid::Uuid::new_v4(),
            rows: RowIter(RefCell::new(Some(rows))),
        };

        self.inner
            .post(url, payload)
            .await?
            .json()
            .await
            .map_err(crate::Error::from)
    }
}

impl<'a, D, T> TableClientRef<'a, D, T> {
    #[inline]
    pub(super) const fn from_parts(dataset_name: D, table_name: T, inner: &'a InnerClient) -> Self {
        Self {
            dataset_name,
            table_name,
            inner,
        }
    }

    pub fn builder(&self) -> schema_builder::TableBuilder<'_, &D, &T> {
        schema_builder::TableBuilder::init(self.inner, &self.dataset_name, &self.table_name)
    }
}

macro_rules! impl_table_methods {
    ($t:ident<$($lt:lifetime,)? $ds_name:ident, $table_name:ident>) => {
        impl<$ds_name, $table_name> $t<$($lt,)? $ds_name, $table_name>
        where
            $ds_name: super::Identifier,
            $table_name: super::Identifier,
        {
            pub async fn get(&self) -> crate::Result<Table> {
                let url = route!(self.inner; "datasets" self.dataset_name "tables" self.table_name);

                let table = self.inner.get(url).await?.json().await?;
                Ok(table)
            }
            pub async fn delete(&self) -> crate::Result<()> {
                let url = route!(self.inner; "datasets" self.dataset_name "tables" self.table_name);

                self.inner.delete(url).await?;
                Ok(())
            }
        }
    };
    ($($t:ident<$($lt:lifetime,)? $ds_name:ident, $table_name:ident>),+ $(,)?) => {
        $(
            impl_table_methods!($t<$($lt,)? $ds_name, $table_name>);
        )+
    }
}

impl_table_methods!(TableClient<D, T>, TableClientRef<'_, D, T>);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase", bound = "")]
struct InsertRows<R>
where
    R: IntoIterator,
    R::Item: serde::Serialize,
{
    #[serde(skip_serializing_if = "is_false")]
    skip_invalid_rows: bool,
    #[serde(skip_serializing_if = "is_false")]
    ignore_unknown_values: bool,
    trace_id: uuid::Uuid,
    rows: RowIter<R>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

struct RowIter<R>(RefCell<Option<R>>);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RowWrapper<R> {
    insert_id: uuid::Uuid,
    json: R,
}

impl<R> serde::Serialize for RowIter<R>
where
    R: IntoIterator,
    R::Item: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let rows = self.0.take().expect("serialize called twice").into_iter();
        let (low, high) = rows.size_hint();

        let len_hint = match high {
            Some(high) => Some(high),
            _ if low > 0 => Some(low),
            _ => None,
        };
        let mut seq_ser = serializer.serialize_seq(len_hint)?;

        for json in rows {
            seq_ser.serialize_element(&RowWrapper {
                json,
                insert_id: uuid::Uuid::new_v4(),
            })?;
        }

        seq_ser.end()
    }
}
