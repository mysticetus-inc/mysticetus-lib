//! [`ConnectionParts`], the underlying read/write interface for spanner.
//!
//! Used by [`Session`] and [`Transaction`] under the hood.
//!
//! TODO: investigate a type safe way to handle using + beginning transactions, both read only and
//! read-write. Too many possible panics in this code that stem from misuse.
//!
//! [`Session`]: crate::Session
//! [`Transaction`]: crate::tx::Transaction
#![allow(dead_code)]

use std::sync::Arc;

use bytes::Bytes;
use gcp_auth_provider::service::AuthSvc;
use protos::spanner::transaction_options::read_only::TimestampBound;
use protos::spanner::transaction_options::read_write::ReadLockMode;
use protos::spanner::transaction_options::{IsolationLevel, Mode, ReadOnly, ReadWrite};
use protos::spanner::{
    self, BeginTransactionRequest, CommitRequest, CommitResponse, ExecuteSqlRequest,
    PartialResultSet, ReadRequest, ResultSet, TransactionOptions, TransactionSelector,
    commit_request, execute_sql_request, read_request,
};

use crate::key_set::KeySet;
use crate::queryable::Queryable;
use crate::results::{ResultIter, StreamingRead};
use crate::sql::Params;
use crate::tx::ReadWriteTx;
use crate::{Table, tx};

/// A const proto key set to read an entire table
const ALL_KEY_SET: spanner::KeySet = spanner::KeySet {
    all: true,
    keys: Vec::new(),
    ranges: Vec::new(),
};

/// Default set of options for a read write transaction.
const DEFAULT_READ_WRITE: TransactionOptions = TransactionOptions {
    isolation_level: IsolationLevel::Unspecified as i32,
    exclude_txn_from_change_streams: false,
    mode: Some(Mode::ReadWrite(ReadWrite {
        multiplexed_session_previous_transaction_id: Bytes::new(),
        read_lock_mode: ReadLockMode::Pessimistic as i32,
    })),
};

/// Default set of options for a read only/snapshot transaction.
const DEFAULT_READ_ONLY: TransactionOptions = TransactionOptions {
    exclude_txn_from_change_streams: false,
    isolation_level: IsolationLevel::Unspecified as i32,
    mode: Some(Mode::ReadOnly(ReadOnly {
        return_read_timestamp: true,
        timestamp_bound: Some(TimestampBound::Strong(true)),
    })),
};

/// Helper type that contains everything needed to make a request,
/// and provides internal helper functions to simplify things internally.
pub struct ConnectionParts<'a, T> {
    pub(crate) client_parts: &'a crate::client::ClientParts,
    pub(crate) session: &'a Arc<Session>,
    tx: T,
}

impl<'a, T> ConnectionParts<'a, T> {
    #[inline]
    pub(crate) fn from_parts(
        client_parts: &'a crate::client::ClientParts,
        session: &'a Arc<Session>,
        tx: T,
    ) -> Self {
        Self {
            client_parts,
            session,
            tx,
        }
    }
}

/// Helper to construct the [`ReadRequest`] used by all Spanner reads (both streaming and
/// unary). Specifically Non-generic.
#[inline]
fn build_read_request(
    session: String,
    tx: TransactionSelector,
    key_set: spanner::KeySet,
    table: String,
    cols: Vec<String>,
    lim: Option<u32>,
    order_by: Option<read_request::OrderBy>,
    lock_hint: Option<read_request::LockHint>,
) -> ReadRequest {
    ReadRequest {
        session,
        data_boost_enabled: false,
        transaction: Some(tx),
        index: String::new(),
        order_by: order_by.unwrap_or(read_request::OrderBy::Unspecified) as i32,
        lock_hint: lock_hint.unwrap_or(read_request::LockHint::Unspecified) as i32,
        directed_read_options: None,
        key_set: Some(key_set),
        table,
        columns: cols,
        limit: lim.unwrap_or(0) as i64,
        resume_token: Bytes::new(),
        partition_token: Bytes::new(),
        request_options: None,
    }
}

/// Helper to construct the [`ExecuteSqlRequest`] used by all Spanner sql request (both streaming
/// and unary). Specifically Non-generic.
#[inline]
fn build_sql_request(
    session: String,
    tx: TransactionSelector,
    sql: String,
    params: Option<Params>,
) -> ExecuteSqlRequest {
    let (params, param_types) = params.map(Params::into_parts).unwrap_or_default();

    ExecuteSqlRequest {
        session,
        data_boost_enabled: false,
        transaction: Some(tx),
        directed_read_options: None,
        resume_token: Bytes::new(),
        partition_token: Bytes::new(),
        request_options: None,
        sql,
        params,
        param_types,
        query_mode: execute_sql_request::QueryMode::Profile as i32,
        seqno: 0,
        query_options: None,
        last_statement: false,
    }
}

impl<Tx> ConnectionParts<'_, Tx> {
    pub(crate) fn client(
        &self,
    ) -> spanner::spanner_client::SpannerClient<AuthSvc<tonic::transport::Channel>> {
        spanner::spanner_client::SpannerClient::new(self.client_parts.channel.clone())
    }
}

// ------------ Read only functions -------------- //
impl<Tx> ConnectionParts<'_, Tx>
where
    Tx: tx::ReadOnlyTx,
{
    // -------------- Read + Streaming Read Inner ---------------- //

    pub(crate) async fn read_inner(
        &mut self,
        key_set: spanner::KeySet,
        table: String,
        cols: Vec<String>,
        lim: Option<u32>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<ResultSet> {
        let raw = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let request = build_read_request(
            raw.name.clone(),
            self.tx.build_read_only_selector(),
            key_set,
            table,
            cols,
            lim,
            order_by,
            lock_hint,
        );

        self.client()
            .read(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(crate::Error::from)
    }

    pub(crate) async fn execute_sql_inner(
        &mut self,
        sql: String,
        params: Option<Params>,
    ) -> crate::Result<ResultSet> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let request = build_sql_request(
            raw_session.name.clone(),
            self.tx.build_read_only_selector(),
            sql,
            params,
        );

        self.client()
            .execute_sql(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(crate::Error::from)
    }

    pub(crate) async fn streaming_read_inner(
        &mut self,
        key_set: spanner::KeySet,
        table: String,
        cols: Vec<String>,
        lim: Option<u32>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<tonic::Streaming<PartialResultSet>> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let req = build_read_request(
            raw_session.name.clone(),
            self.tx.build_read_only_selector(),
            key_set,
            table,
            cols,
            lim,
            order_by,
            lock_hint,
        );

        self.client()
            .streaming_read(req)
            .await
            .map(tonic::Response::into_inner)
            .map_err(crate::Error::from)
    }

    pub(crate) async fn execute_streaming_sql_inner(
        &mut self,
        sql: String,
        params: Option<Params>,
    ) -> crate::Result<tonic::Streaming<PartialResultSet>> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let request = build_sql_request(
            raw_session.name.clone(),
            self.tx.build_read_only_selector(),
            sql,
            params,
        );

        self.client()
            .execute_streaming_sql(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(crate::Error::from)
    }

    // ------------ Read + Streaming Read Table -------------- //

    pub(crate) async fn read_table<T: Table + Queryable>(
        &mut self,
        key_set: spanner::KeySet,
        lim: Option<u32>,
    ) -> crate::Result<ResultIter<T>> {
        let table = T::NAME.to_owned();
        let cols = crate::util::table_col_names::<T>();

        let rs = self
            .read_inner(key_set, table, cols, lim, None, None)
            .await?;

        ResultIter::from_result_set(rs)
    }

    pub(crate) async fn streaming_read_table<T: Table + Queryable>(
        &mut self,
        key_set: spanner::KeySet,
        lim: Option<u32>,
    ) -> crate::Result<StreamingRead<T>> {
        let table = T::NAME.to_owned();
        let cols = crate::util::table_col_names::<T>();

        let streaming = self
            .streaming_read_inner(key_set, table, cols, lim, None, None)
            .await?;

        Ok(StreamingRead::from_streaming(streaming))
    }

    // --------- Read + Streaming Read from KeySet ------------ //

    pub(crate) async fn read_key_set<T: Table + Queryable>(
        &mut self,
        key_set: KeySet<T>,
    ) -> crate::Result<ResultIter<T>> {
        let lim = key_set.get_limit();
        let key_set = key_set.into_proto();
        self.read_table(key_set, lim).await
    }

    pub(crate) async fn streaming_read_key_set<T: Table + Queryable>(
        &mut self,
        key_set: KeySet<T>,
    ) -> crate::Result<StreamingRead<T>> {
        let lim = key_set.get_limit();
        let key_set = key_set.into_proto();
        self.streaming_read_table(key_set, lim).await
    }

    // ------------- Read Entire Table ----------- //

    #[inline]
    pub(crate) async fn streaming_read_all<T: Table + Queryable>(
        &mut self,
    ) -> crate::Result<StreamingRead<T>> {
        self.streaming_read_table::<T>(ALL_KEY_SET, None).await
    }

    #[inline]
    pub(crate) async fn read_all<T: Table + Queryable>(&mut self) -> crate::Result<ResultIter<T>> {
        self.read_table::<T>(ALL_KEY_SET, None).await
    }

    // -------------- Read from 1 or more PK --------------- //

    #[inline]
    pub(crate) async fn read_one<T: Table + Queryable>(
        &mut self,
        pk: T::Pk,
    ) -> crate::Result<Option<T>> {
        let mut key_set = KeySet::<T>::with_capacity(1, 0);
        key_set.add_key(pk);

        let mut result_set = self.read_key_set(key_set).await?;

        // sanity check that a key set with one key in it returns 0 or 1 row.
        debug_assert!(result_set.len() < 2);

        match result_set.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(error)) => Err(error),
            None => Ok(None),
        }
    }

    #[inline]
    pub(crate) async fn read_rows<T: Table + Queryable, I>(
        &mut self,
        pks: I,
    ) -> crate::Result<ResultIter<T>>
    where
        I: IntoIterator<Item = T::Pk>,
    {
        let key_set = KeySet::<T>::from_iter(pks);

        self.read_key_set(key_set).await
    }

    #[inline]
    pub(crate) async fn streaming_read_rows<T: Table + Queryable, I>(
        &mut self,
        pks: I,
    ) -> crate::Result<StreamingRead<T>>
    where
        I: IntoIterator<Item = T::Pk>,
    {
        let key_set = KeySet::<T>::from_iter(pks);

        self.streaming_read_key_set(key_set).await
    }
}

#[inline]
fn build_commit_request(
    session: String,
    tx: commit_request::Transaction,
    mutations: Vec<spanner::Mutation>,
) -> CommitRequest {
    CommitRequest {
        max_commit_delay: None,
        session,
        mutations,
        return_commit_stats: true,
        request_options: None,
        transaction: Some(tx),
        precommit_token: None,
    }
}

// ----------- read/write Transaction specific ----------- //

impl<Tx: ReadWriteTx> ConnectionParts<'_, Tx> {
    #[inline]
    pub(crate) async fn commit_inner(
        &mut self,
        mutations: Vec<spanner::Mutation>,
    ) -> crate::Result<tonic::Response<CommitResponse>> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let request = build_commit_request(
            raw_session.name.clone(),
            self.tx.build_read_write(),
            mutations,
        );

        self.client()
            .commit(request)
            .await
            .map_err(crate::Error::Status)
    }
}

// ----------- Existing, Read-write Transaction specific ----------- //
impl ConnectionParts<'_, tx::Existing<'_, tx::ReadWrite>> {
    pub(crate) async fn rollback(&mut self) -> crate::Result<()> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let request = spanner::RollbackRequest {
            session: raw_session.name.clone(),
            transaction_id: self.tx.clone_id(),
        };

        self.client().rollback(request).await?;

        Ok(())
    }
}

// ----------- Special begin tx methods --------------- //
impl<'a, Tx: tx::TxOptions + Copy> ConnectionParts<'a, tx::Begin<Tx>> {
    /// Begins a standalone transaction, not tied to a specific read.
    pub(crate) async fn begin_tx(&mut self) -> crate::Result<spanner::Transaction> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let request = BeginTransactionRequest {
            session: raw_session.name.clone(),
            options: Some(Tx::OPTIONS),
            request_options: None,
            mutation_key: None,
        };

        self.client()
            .begin_transaction(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(crate::Error::from)
    }

    /// Start a transaction, and do a read at the same time (within the started transaction).
    pub(crate) async fn begin_tx_read_inner(
        &mut self,
        key_set: spanner::KeySet,
        table: String,
        cols: Vec<String>,
        lim: Option<u32>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<(spanner::Transaction, ResultSet)> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let req = build_read_request(
            raw_session.name.clone(),
            self.tx.build_selector(),
            key_set,
            table,
            cols,
            lim,
            order_by,
            lock_hint,
        );

        let mut result_set = self.client().read(req).await?.into_inner();

        let tx = result_set
            .metadata
            .as_mut()
            .and_then(|meta| meta.transaction.take())
            .ok_or_else(|| anyhow::anyhow!("transaction not started"))?;

        Ok((tx, result_set))
    }

    /// Start a transaction, and do a streaming read at the same time (within the started
    /// transaction).
    pub(crate) async fn begin_tx_streaming_read_inner(
        &mut self,
        key_set: spanner::KeySet,
        table: String,
        cols: Vec<String>,
        lim: Option<u32>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<(
        spanner::Transaction,
        PartialResultSet,
        tonic::Streaming<PartialResultSet>,
    )> {
        let raw_session = self
            .session
            .raw_session()
            .ok_or(crate::Error::SessionDeleted)?;

        let req = build_read_request(
            raw_session.name.clone(),
            self.tx.build_selector(),
            key_set,
            table,
            cols,
            lim,
            order_by,
            lock_hint,
        );

        let mut streaming = self.client().streaming_read(req).await?.into_inner();

        let mut first_chunk = streaming
            .message()
            .await?
            .ok_or(crate::Error::MissingResultMetadata)?;

        let tx = first_chunk
            .metadata
            .as_mut()
            .and_then(|meta| meta.transaction.take())
            .ok_or_else(|| anyhow::anyhow!("no transaction recieved"))?;

        Ok((tx, first_chunk, streaming))
    }

    pub(crate) async fn begin_tx_read_table<T: Table + Queryable>(
        &mut self,
        key_set: KeySet<T>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<(spanner::Transaction, ResultIter<T>)> {
        let lim = key_set.get_limit();
        let key_set = key_set.into_proto();
        let table = T::NAME.to_owned();
        let cols = crate::util::table_col_names::<T>();
        let (tx, result_set) = self
            .begin_tx_read_inner(key_set, table, cols, lim, order_by, lock_hint)
            .await?;

        let iter = ResultIter::from_result_set(result_set)?;

        Ok((tx, iter))
    }

    pub(crate) async fn begin_tx_streaming_read_table<T: Table + Queryable>(
        &mut self,
        key_set: KeySet<T>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<(spanner::Transaction, StreamingRead<T>)> {
        let lim = key_set.get_limit();
        let key_set = key_set.into_proto();
        let table = T::NAME.to_owned();
        let cols = crate::util::table_col_names::<T>();
        let (tx, first_chunk, streaming) = self
            .begin_tx_streaming_read_inner(key_set, table, cols, lim, order_by, lock_hint)
            .await?;

        let iter = StreamingRead::new_with_first_chunk(streaming, first_chunk)?;

        Ok((tx, iter))
    }
}

// ------------ Specific begin read-write tx functions ------------- //
impl<'a> ConnectionParts<'a, tx::Begin<tx::ReadWrite>> {
    pub(crate) async fn begin_tx_read<T: Table + Queryable>(
        mut self,
        key_set: KeySet<T>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<(crate::tx::Transaction<'a>, ResultIter<T>)> {
        let (tx, iter) = self
            .begin_tx_read_table::<T>(key_set, order_by, lock_hint)
            .await?;

        Ok((crate::tx::Transaction::new(self, tx), iter))
    }

    pub(crate) async fn begin_tx_streaming_read<T: Table + Queryable>(
        mut self,
        key_set: KeySet<T>,
        order_by: Option<read_request::OrderBy>,
        lock_hint: Option<read_request::LockHint>,
    ) -> crate::Result<(crate::tx::Transaction<'a>, StreamingRead<T>)> {
        let (tx, iter) = self
            .begin_tx_streaming_read_table(key_set, order_by, lock_hint)
            .await?;

        Ok((crate::tx::Transaction::new(self, tx), iter))
    }
}

macro_rules! impl_deferred_read_functions {
    () => {
        #[inline]
        pub async fn read<T, K>(&self, key_set: K) -> $crate::Result<$crate::results::ResultIter<T>>
        where
            T: $crate::Table + $crate::queryable::Queryable,
            K: $crate::key_set::IntoKeySet<T>,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .read_key_set(key_set.into_key_set())
                .await
        }

        #[inline]
        pub async fn streaming_read<T, K>(
            &self,
            key_set: K,
        ) -> $crate::Result<$crate::results::StreamingRead<T>>
        where
            T: $crate::Table + $crate::queryable::Queryable,
            K: $crate::key_set::IntoKeySet<T>,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .streaming_read_key_set(key_set.into_key_set())
                .await
        }

        #[inline]
        pub async fn read_all<T>(&self) -> $crate::Result<$crate::results::ResultIter<T>>
        where
            T: $crate::Table + $crate::queryable::Queryable,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .read_all()
                .await
        }

        #[inline]
        pub async fn streaming_read_all<T>(
            &self,
        ) -> $crate::Result<$crate::results::StreamingRead<T>>
        where
            T: $crate::Table + $crate::queryable::Queryable,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .streaming_read_all()
                .await
        }

        #[inline]
        pub async fn read_one<T>(&self, pk: T::Pk) -> $crate::Result<Option<T>>
        where
            T: $crate::Table + $crate::queryable::Queryable,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .read_one(pk)
                .await
        }

        #[inline]
        #[allow(dead_code)]
        pub(crate) async fn read_rows<T: $crate::Table, I>(
            &self,
            pks: I,
        ) -> $crate::Result<$crate::results::ResultIter<T>>
        where
            I: IntoIterator<Item = T::Pk>,
            T: $crate::Table + $crate::queryable::Queryable,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .read_rows(pks)
                .await
        }

        #[allow(dead_code)]
        #[inline]
        pub(crate) async fn streaming_read_rows<T: $crate::Table, I>(
            &self,
            pks: I,
        ) -> $crate::Result<$crate::results::StreamingRead<T>>
        where
            I: IntoIterator<Item = T::Pk>,
            T: $crate::Table + $crate::queryable::Queryable,
        {
            <Self as $crate::private::SealedConnection>::connection_parts(&self)
                .streaming_read_rows(pks)
                .await
        }
    };
}

pub(crate) use impl_deferred_read_functions;

use super::pool::Session;
