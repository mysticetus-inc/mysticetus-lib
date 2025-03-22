use std::sync::Arc;

use protos::spanner::commit_response::CommitStats;
use timestamp::Timestamp;
use tokio::task::JoinHandle;

use super::ClientParts;
use super::connection::ConnectionParts;
use super::pool::Session;
use crate::error::ConvertError;
use crate::key_set::{IntoKeySet, KeySet, OwnedRangeBounds, WriteBuilder};
use crate::pk::IntoPartialPkParts;
use crate::private::SealedConnection;
use crate::queryable::Queryable;
use crate::tx::{ShouldCommit, SingleUse, Transaction};
use crate::{ResultIter, StreamingRead, Table};

pub struct SessionClient {
    pub(super) parts: Arc<ClientParts>,
    pub(super) session: Arc<Session>,
}

impl Drop for SessionClient {
    fn drop(&mut self) {
        use tonic::Code;

        use crate::Error;

        if let Some(handle) = self.return_session() {
            tokio::spawn(async move {
                match handle.await {
                    Ok(Ok(())) => (),
                    // if a session was deleted by spanner itself, we'll get a not found when we try
                    // to delete it again. Ignore, since the end result is what we wanted anyways
                    Ok(Err(Error::Status(status))) if status.code() == Code::NotFound => (),
                    Ok(Err(error)) => {
                        tracing::error!(message = "failed to delete expired session", ?error);
                    }
                    // ignore cancelled errors
                    Err(error) if error.is_cancelled() => (),
                    Err(error) => {
                        tracing::error!(message = "panic trying to delete expired session", ?error);
                    }
                }
            });
        }
    }
}

impl SealedConnection for SessionClient {
    type Tx<'a> = SingleUse;

    #[inline]
    fn connection_parts(&self) -> ConnectionParts<'_, Self::Tx<'_>> {
        ConnectionParts::from_parts(&self.parts, &self.session, crate::tx::SingleUse)
    }
}
impl SessionClient {
    crate::client::connection::impl_deferred_read_functions!();

    fn return_session(&self) -> Option<JoinHandle<crate::Result<()>>> {
        super::SESSION_POOL.return_session(&self.parts, &self.session)
    }

    pub fn session_role(&self) -> Option<&str> {
        self.parts.role.as_deref()
    }

    async fn write_inner(
        &self,
        mutations: Vec<protos::spanner::Mutation>,
    ) -> crate::Result<Option<CommitStats>> {
        let resp = self
            .connection_parts()
            .commit_inner(mutations)
            .await?
            .into_inner();

        Ok(resp.commit_stats)
    }

    pub async fn insert_or_update<T: crate::Table>(
        &self,
        rows: WriteBuilder<T>,
    ) -> crate::Result<Option<CommitStats>> {
        let mutation = protos::spanner::Mutation {
            operation: Some(protos::spanner::mutation::Operation::InsertOrUpdate(
                rows.into_proto(),
            )),
        };

        self.write_inner(vec![mutation]).await
    }
    pub async fn run_in_transaction<F, Fut>(&self, mut func: F) -> crate::Result<Option<Timestamp>>
    where
        F: FnMut(&mut Transaction<'_>) -> Fut,
        Fut: std::future::Future<Output = crate::Result<ShouldCommit>>,
    {
        let mut tx = self.begin_transaction().await?;

        match func(&mut tx).await {
            Ok(ShouldCommit::Yes) => {
                let resp = tx.commit().await?;
                Ok(resp.commit_timestamp.map(Timestamp::from))
            }
            Ok(ShouldCommit::No) => {
                tx.rollback().await?;
                Ok(None)
            }
            Err(error) => {
                // if rolling back fails, there's likely a transport issue we can't do
                // anything about. If that happens, just log and return the original error.
                if let Err(tx_error) = tx.rollback().await {
                    error!(message = "error rolling back transaction", ?tx_error, orig_error = ?error);
                }

                return Err(error);
            }
        }
    }

    pub async fn begin_transaction(&self) -> crate::Result<Transaction<'_>> {
        let mut parts =
            ConnectionParts::from_parts(&self.parts, &self.session, crate::tx::Begin::default());

        let tx = parts.begin_tx().await?;

        Ok(Transaction::new(parts, tx))
    }

    pub(crate) async fn execute_dml(
        &self,
        statements: Vec<protos::spanner::execute_batch_dml_request::Statement>,
    ) -> crate::Result<protos::spanner::ExecuteBatchDmlResponse> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let req = protos::spanner::ExecuteBatchDmlRequest {
            session: raw_session.name.clone(),
            transaction: None,
            statements,
            seqno: 0,
            request_options: None,
        };

        let resp = protos::spanner::spanner_client::SpannerClient::new(self.parts.channel.clone())
            .execute_batch_dml(req)
            .await?
            .into_inner();

        Ok(resp)
    }

    pub async fn execute_streaming_sql<T: Queryable>(
        &self,
        sql: String,
        params: Option<crate::sql::Params>,
    ) -> crate::Result<StreamingRead<T>> {
        let streaming = self
            .connection_parts()
            .execute_streaming_sql_inner(sql, params)
            .await?;

        Ok(StreamingRead::from_streaming(streaming))
    }

    pub async fn execute_sql<T: Queryable>(
        &self,
        sql: String,
        params: Option<crate::sql::Params>,
    ) -> crate::Result<ResultIter<T>> {
        let result_set = self
            .connection_parts()
            .execute_sql_inner(sql, params)
            .await?;

        ResultIter::from_result_set(result_set)
    }

    pub fn mutate(&self, capacity: usize) -> MutationBuilder<'_> {
        MutationBuilder {
            mutations: Vec::with_capacity(capacity),
            client: self,
        }
    }

    pub(crate) async fn refresh_session(&self) -> crate::Result<()> {
        let res = self
            .connection_parts()
            .execute_sql_inner("SELECT 1".into(), None)
            .await?;

        #[cfg(debug_assertions)]
        {
            use protos::protobuf::value::Kind::NumberValue;
            assert_eq!(res.rows.len(), 1, "'SELECT 1' should only return 1 row");
            let row = &res.rows[0].values;
            assert_eq!(row.len(), 1, "`SELECT 1` row should only have one column");
            let kind = row[0].kind.as_ref().expect("should exist");
            assert_eq!(*kind, NumberValue(1.0));
        }

        let _ = res;

        Ok(())
    }
}

pub struct MutationBuilder<'a> {
    mutations: Vec<protos::spanner::Mutation>,
    client: &'a SessionClient,
}

macro_rules! impl_methods {
    ($(
        [$main_method:ident, $one_method:ident, $variant:ident]
    ),* $(,)?) => {
        $(
            pub fn $main_method<T: crate::Table>(&mut self, write_builder: WriteBuilder<T>) -> &mut Self {
                self.append_mutation(protos::spanner::Mutation {
                    operation: Some(protos::spanner::mutation::Operation::$variant(
                        write_builder.into_proto(),
                    )),
                })
            }

            pub fn $one_method<T: crate::Table>(&mut self, row: T) -> Result<&mut Self, ConvertError> {
                let mut builder = WriteBuilder::with_row_capacity(1);
                builder.add_row(row)?;
                Ok(self.$main_method(builder))
            }
        )*
    };
}

impl<'a> MutationBuilder<'a> {
    pub async fn commit(&mut self) -> crate::Result<Option<CommitStats>> {
        let mutations = std::mem::take(&mut self.mutations);
        self.client.write_inner(mutations).await
    }

    fn append_mutation(&mut self, mutation: protos::spanner::Mutation) -> &mut Self {
        self.mutations.push(mutation);
        self
    }

    pub fn delete_row<T: Table>(&mut self, pk: T::Pk) -> &mut Self {
        self.delete_rows::<T>([pk])
    }

    pub fn delete_rows<T: Table>(&mut self, rows: impl IntoIterator<Item = T::Pk>) -> &mut Self {
        let iter = rows.into_iter();
        let (low, high) = iter.size_hint();
        let mut ks = KeySet::<T>::with_capacity(high.unwrap_or(low), 0);
        ks.add_keys(iter);
        self.delete::<T>(ks)
    }

    pub fn delete_range<T: Table, R, B>(&mut self, range: R) -> &mut Self
    where
        R: OwnedRangeBounds<B>,
        B: IntoPartialPkParts<T>,
    {
        let mut ks = KeySet::<T>::with_capacity(0, 1);
        ks.add_range(range);
        self.delete(ks)
    }

    pub fn delete<T: Table>(&mut self, to_delete: impl IntoKeySet<T>) -> &mut Self {
        self.append_mutation(protos::spanner::Mutation {
            operation: Some(protos::spanner::mutation::Operation::Delete(
                protos::spanner::mutation::Delete {
                    table: T::NAME.to_owned(),
                    key_set: Some(to_delete.into_key_set().into_proto()),
                },
            )),
        })
    }

    impl_methods! {
        [insert_or_update, insert_or_update_one, InsertOrUpdate],
        [update, update_one, Update],
        [replace, replace_one, Replace],
        [insert, insert_one, Insert],
    }
}
