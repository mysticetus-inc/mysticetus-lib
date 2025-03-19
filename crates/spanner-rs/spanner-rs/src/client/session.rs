use std::sync::Arc;

use protos::spanner::commit_response::CommitStats;
use timestamp::Timestamp;

use super::ClientParts;
use super::connection::{ConnectionParts, RawSession};
use crate::error::ConvertError;
use crate::key_set::WriteBuilder;
use crate::private::SealedConnection;
use crate::tx::{ShouldCommit, Transaction};
use crate::{ResultIter, StreamingRead};

pub struct SessionClient {
    pub(super) parts: Arc<ClientParts>,
    pub(super) session: super::pool::BorrowedSession,
}

impl<'session> SealedConnection<'session> for &'session SessionClient {
    type Tx<'a>
        = crate::tx::SingleUse
    where
        'session: 'a;

    #[inline]
    fn connection_parts(&self) -> ConnectionParts<'_, 'session, Self::Tx<'_>> {
        ConnectionParts::from_parts(
            &self.parts,
            RawSession::Pending(&self.session),
            crate::tx::SingleUse,
        )
    }
}
impl SessionClient {
    crate::client::connection::impl_deferred_read_functions!();

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
        F: FnMut(&mut Transaction<'_, '_>) -> Fut,
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

    pub async fn begin_transaction(&self) -> crate::Result<Transaction<'_, '_>> {
        let mut parts = ConnectionParts::from_parts(
            &self.parts,
            RawSession::Pending(&self.session),
            crate::tx::Begin::default(),
        );

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
            .await
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

    pub async fn execute_streaming_sql<T: crate::Table>(
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

    pub async fn execute_sql<T: crate::Table>(
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

    impl_methods! {
        [insert_or_update, insert_or_update_one, InsertOrUpdate],
        [update, update_one, Update],
        [replace, replace_one, Replace],
        [insert, insert_one, Insert],
    }
}
