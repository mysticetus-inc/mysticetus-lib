use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio_util::sync::ReusableBoxFuture;

use super::InnerClient;
use crate::resources::table::Table;

pin_project_lite::pin_project! {
    #[project = TableStreamProjection]
    pub struct TableStream<'a, 'c, D> {
        client: &'a super::DatasetClient<'c, D>,
        done: bool,
        max_page_size: NonZeroUsize,
        buf: Vec<Table>,
        request_fut: ReusableBoxFuture<'a, crate::Result<TablePaginationWrapper>>,
    }
}

impl<'a, 'c, D: AsRef<str>> TableStream<'a, 'c, D> {
    pub(super) fn new(client: &'a super::DatasetClient<'c, D>, max_page_size: usize) -> Self {
        let max_page_size = NonZeroUsize::new(max_page_size).unwrap_or(NonZeroUsize::MIN);

        TableStream {
            client,
            done: false,
            buf: Vec::new(),
            max_page_size,
            request_fut: ReusableBoxFuture::new(req_table_list(
                &client.client,
                &client.dataset_name,
                max_page_size,
                None,
            )),
        }
    }
}

fn req_table_list<'a, D: ?Sized + AsRef<str>>(
    inner: &'a InnerClient,
    dataset_name: &D,
    max_page_size: NonZeroUsize,
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

        crate::client::deserialize_json(resp).await
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TablePaginationWrapper {
    next_page_token: Option<Box<str>>,
    tables: Vec<Table>,
}

impl<D: AsRef<str>> TableStream<'_, '_, D> {
    pub async fn collect(self) -> crate::Result<Vec<Table>> {
        let pinned = std::pin::pin!(self);
        let mut this = pinned.project();

        while !*this.done {
            std::future::poll_fn(|cx| this.poll_drive(cx)).await?;
        }

        Ok(std::mem::take(this.buf))
    }
}

impl<D: AsRef<str>> TableStreamProjection<'_, '_, '_, D> {
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
                        &self.client.client,
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

impl<D: AsRef<str>> futures::Stream for TableStream<'_, '_, D> {
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
