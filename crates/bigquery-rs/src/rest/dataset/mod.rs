use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio_util::sync::ReusableBoxFuture;

use super::client::InnerClient;
use super::route;
use super::table::TableClient;
use crate::rest::resources::table::Table;

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

    pub fn client(&self) -> super::BigQueryClient {
        super::BigQueryClient {
            inner: Arc::clone(&self.inner),
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
}
impl<D: AsRef<str>> DatasetClient<D> {
    pub fn list_tables(&self, max_page_size: usize) -> TableStream<'_, D> {
        TableStream {
            client: self,
            done: false,
            buf: Vec::new(),
            max_page_size,
            request_fut: ReusableBoxFuture::new(req_table_list(
                &self.inner,
                &self.dataset_name,
                max_page_size,
                None,
            )),
        }
    }

    pub async fn create_table(&self, table: Table) -> crate::Result<Table>
    where
        D: super::Identifier,
    {
        let url = route!(self.inner; "datasets" self.dataset_name "tables");
        let resp = self.inner.post(url, table).await?;
        super::client::deserialize_json(resp).await
    }
}

fn req_table_list<'a, D: ?Sized + AsRef<str>>(
    inner: &'a InnerClient,
    dataset_name: &D,
    max_page_size: usize,
    token: Option<Box<str>>,
) -> impl Future<Output = crate::Result<TablePaginationWrapper>> + Send + 'a {
    let url = inner.make_url(["datasets", dataset_name.as_ref(), "tables"]);

    async move {
        let mut builder = inner.request(reqwest::Method::GET, url).await?;

        if let Some(token) = token {
            builder = builder.query(&[("pageToken", &token)]);
        }

        let resp = builder
            .query(&[("maxResults", max_page_size)])
            .send()
            .await?
            .error_for_status()?;

        super::client::deserialize_json(resp).await
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TablePaginationWrapper {
    next_page_token: Option<Box<str>>,
    tables: Vec<Table>,
}

pin_project_lite::pin_project! {
    #[project = TableStreamProjection]
    pub struct TableStream<'a, D> {
        client: &'a DatasetClient<D>,
        done: bool,
        max_page_size: usize,
        buf: Vec<Table>,
        request_fut: ReusableBoxFuture<'a, crate::Result<TablePaginationWrapper>>,
    }
}

impl<D: AsRef<str>> TableStream<'_, D> {
    pub async fn collect(self) -> crate::Result<Vec<Table>> {
        let pinned = std::pin::pin!(self);
        let mut this = pinned.project();

        while !*this.done {
            std::future::poll_fn(|cx| this.poll_drive(cx)).await?;
        }

        Ok(std::mem::take(this.buf))
    }
}

impl<D: AsRef<str>> TableStreamProjection<'_, '_, D> {
    fn poll_drive(&mut self, cx: &mut Context<'_>) -> Poll<crate::Result<()>> {
        loop {
            let result = std::task::ready!(self.request_fut.poll(cx));

            let TablePaginationWrapper {
                next_page_token,
                mut tables,
            } = result?;

            if self.buf.is_empty() {
                *self.buf = tables;
            } else {
                self.buf.append(&mut tables);
            }

            match next_page_token {
                Some(next_page_token) if !next_page_token.is_empty() => {
                    // we need to poll again, so keep looping.
                    self.request_fut.set(req_table_list(
                        &self.client.inner,
                        &self.client.dataset_name,
                        *self.max_page_size,
                        Some(next_page_token),
                    ));
                }
                _ => {
                    *self.done = true;
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}

impl<D: AsRef<str>> futures::Stream for TableStream<'_, D> {
    type Item = crate::Result<Vec<Table>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            if *this.done && this.buf.is_empty() {
                return Poll::Ready(None);
            } else if *this.done {
                return Poll::Ready(Some(Ok(std::mem::take(this.buf))));
            }

            match this.poll_drive(cx) {
                Poll::Pending if this.buf.is_empty() => return Poll::Pending,
                Poll::Pending => return Poll::Ready(Some(Ok(std::mem::take(this.buf)))),
                Poll::Ready(result) => result?,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_table_list() -> crate::Result<()> {
        let dataset_client = super::super::BigQueryClient::new(
            "mysticetus-boem",
            gcp_auth_channel::Scope::BigQueryReadOnly,
        )
        .await?
        .dataset("main");

        let tables = dataset_client.list_tables(10).collect().await?;
        println!("{tables:#?}");

        let serialized_tables = serde_json::to_string_pretty(&tables).unwrap();
        tokio::fs::write("tables.json", serialized_tables.as_bytes())
            .await
            .unwrap();
        Ok(())
    }
}
