use bytes::Bytes;
use gcp_auth_channel::AuthChannel;
use protos::spanner::commit_response::CommitStats;
use protos::spanner::spanner_client::SpannerClient;
use protos::spanner::transaction_options::read_write::ReadLockMode;
use protos::spanner::transaction_options::Mode;
use protos::spanner::{
    self, transaction_options, BeginTransactionRequest, CommitRequest, TransactionOptions,
};
use timestamp::Timestamp;

use crate::connection::impl_deferred_read_functions;
use crate::error::Error;
use crate::key_set::{IntoKeySet, WriteBuilder};
use crate::private::SealedConnection;
use crate::results::{ResultIter, StreamingRead};
use crate::tx::{ShouldCommit, Transaction};
use crate::Table;

#[derive(Debug)]
pub struct Session {
    pub(crate) client: SpannerClient<AuthChannel>,
    pub(crate) session: spanner::Session,
    is_deleted: bool,
}

impl crate::private::SealedConnection for Session {
    type Tx<'a> = crate::tx::SingleUse;

    #[inline]
    fn connection_parts(&self) -> crate::connection::ConnectionParts<'_, Self::Tx<'_>> {
        crate::connection::ConnectionParts::from_parts(self, crate::tx::SingleUse)
    }
}

macro_rules! check_if_deleted {
    ($session:expr $(,)?) => {{
        if $session.is_deleted {
            return Err(crate::Error::SessionDeleted);
        }
    }};
    ($session:expr, $map:expr) => {{
        if $session.is_deleted {
            return Err($map(crate::Error::SessionDeleted));
        }
    }};
}

impl Session {
    impl_deferred_read_functions!();

    #[inline]
    pub(crate) fn new(session: spanner::Session, client: SpannerClient<AuthChannel>) -> Self {
        Self {
            session,
            client,
            is_deleted: false,
        }
    }

    pub fn session_role(&self) -> Option<&str> {
        if self.session.creator_role.is_empty() {
            None
        } else {
            Some(&self.session.creator_role)
        }
    }

    async fn write_inner(
        &self,
        mutations: Vec<spanner::Mutation>,
    ) -> Result<Option<CommitStats>, Error> {
        let req = CommitRequest {
            precommit_token: None,
            session: self.session.name.clone(),
            mutations,
            max_commit_delay: None,
            return_commit_stats: true,
            request_options: None,
            transaction: Some(spanner::commit_request::Transaction::SingleUseTransaction(
                TransactionOptions {
                    exclude_txn_from_change_streams: false,
                    mode: Some(Mode::ReadWrite(spanner::transaction_options::ReadWrite {
                        multiplexed_session_previous_transaction_id: Bytes::new(),
                        read_lock_mode: ReadLockMode::Pessimistic as i32,
                    })),
                },
            )),
        };

        let mut client = self.client.clone();

        let resp = client.commit(req).await?.into_inner();

        Ok(resp.commit_stats)
    }

    pub async fn insert_or_update<T: Table>(
        &mut self,
        rows: WriteBuilder<T>,
    ) -> Result<Option<CommitStats>, Error> {
        self.write_inner(vec![spanner::Mutation {
            operation: Some(spanner::mutation::Operation::InsertOrUpdate(
                rows.into_proto(),
            )),
        }])
        .await
    }
    pub async fn run_in_transaction<F, Fut>(
        &mut self,
        mut func: F,
    ) -> crate::Result<Option<Timestamp>>
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

    pub async fn begin_transaction(&mut self) -> crate::Result<Transaction<'_>> {
        let request = BeginTransactionRequest {
            session: self.session.name.clone(),
            options: Some(TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Some(Mode::ReadWrite(transaction_options::ReadWrite {
                    multiplexed_session_previous_transaction_id: Bytes::new(),
                    read_lock_mode: ReadLockMode::Pessimistic as i32,
                })),
            }),
            request_options: None,
            mutation_key: None,
        };

        let tx = self.client.begin_transaction(request).await?.into_inner();

        Ok(Transaction::new(self, tx))
    }

    pub(crate) async fn execute_dml(
        &mut self,
        statements: Vec<spanner::execute_batch_dml_request::Statement>,
    ) -> crate::Result<spanner::ExecuteBatchDmlResponse> {
        check_if_deleted!(self);

        let req = spanner::ExecuteBatchDmlRequest {
            session: self.session.name.clone(),
            transaction: None,
            statements,
            seqno: 0,
            request_options: None,
        };

        let resp = self.client.execute_batch_dml(req).await?.into_inner();
        Ok(resp)
    }

    pub async fn execute_streaming_sql<T: Table>(
        &mut self,
        sql: String,
        params: Option<crate::sql::Params>,
    ) -> crate::Result<StreamingRead<T>> {
        check_if_deleted!(self);
        let streaming = self
            .connection_parts()
            .execute_streaming_sql_inner(sql, params)
            .await?;

        Ok(StreamingRead::from_streaming(streaming))
    }

    pub async fn execute_sql<T: Table>(
        &mut self,
        sql: String,
        params: Option<crate::sql::Params>,
    ) -> crate::Result<ResultIter<T>> {
        check_if_deleted!(self);
        let result_set = self
            .connection_parts()
            .execute_sql_inner(sql, params)
            .await?;

        ResultIter::from_result_set(result_set)
    }

    pub(crate) async fn refresh_session(&mut self) -> crate::Result<()> {
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

    /// Deletes the session.
    pub async fn delete(&mut self) -> crate::Result<()> {
        if self.is_deleted {
            return Ok(());
        }

        let req = spanner::DeleteSessionRequest {
            name: self.session.name.clone(),
        };
        self.client.delete_session(req).await?;
        self.is_deleted = true;
        Ok(())
    }
}

pub struct WriteRequestBuilder<'a> {
    session: &'a mut Session,
    mutations: Vec<spanner::Mutation>,
}

impl WriteRequestBuilder<'_> {
    pub fn delete_key_set<T: Table>(&mut self, ks: impl IntoKeySet<T>) -> &mut Self {
        self.mutations.push(spanner::Mutation {
            operation: Some(spanner::mutation::Operation::Delete(
                spanner::mutation::Delete {
                    table: T::NAME.to_owned(),
                    key_set: Some(ks.into_key_set().into_proto()),
                },
            )),
        });
        self
    }

    pub fn delete_range<T>(&mut self) -> &mut Self {
        self
    }

    pub async fn commit(self) -> crate::Result<Option<CommitStats>> {
        self.session
            .write_inner(self.mutations)
            .await
            .map_err(crate::Error::from)
    }
}
