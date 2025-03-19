use net_utils::backoff::Backoff;
use protos::spanner::mutation::{Delete, Operation, Write};
use protos::spanner::{self, CommitResponse, Mutation};
use tonic::Code;

use crate::Table;
use crate::client::ClientParts;
use crate::client::connection::{ConnectionParts, RawSession};
use crate::error::ConvertError;
use crate::key_set::{KeySet, WriteBuilder};
use crate::util::MaybeOwnedMut;

#[must_use = "a transaction must be committed or rolled back"]
pub struct Transaction<'a, 'session> {
    raw_session: MaybeOwnedMut<'a, RawSession<'session>>,
    client_parts: &'a ClientParts,
    mutations: Vec<Mutation>,
    tx: spanner::Transaction,
    /// [`None`] is an uncommitted, not rolled-back transaction. When one of those 2 occurs,
    /// this is set (before any errors are thrown).
    state: Option<TxState>,
    /// If 'state' is [`Some`], this incidates that the operation was successful.
    /// If 'state' is [`None`], this has no important meaning.
    completed_successfully: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TxState {
    RolledBack,
    Committed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShouldCommit {
    Yes,
    No,
}

impl<'a, 'session> Transaction<'a, 'session> {
    pub(crate) fn new(
        parts: ConnectionParts<'a, 'session, crate::tx::Begin<crate::tx::ReadWrite>>,
        tx: spanner::Transaction,
    ) -> Self {
        Self {
            mutations: vec![],
            client_parts: parts.client_parts,
            raw_session: parts.raw_session,
            tx,
            state: None,
            completed_successfully: false,
        }
    }

    pub(crate) fn parts(
        &mut self,
    ) -> ConnectionParts<'_, 'session, super::Existing<'_, super::ReadWrite>> {
        crate::client::connection::ConnectionParts::from_parts(
            self.client_parts,
            self.raw_session.reborrow(),
            super::Existing::new(&self.tx),
        )
    }
}

/*
impl<'tx, 'session> crate::private::SealedConnection<'session> for Transaction<'tx, 'session> {
    type Tx<'a>
        = super::Existing<'a, super::ReadWrite>
    where
        Self: 'a;

    #[inline]
    fn connection_parts(&self) -> ConnectionParts<'_, 'session, Self::Tx<'_>> {
        // self.parts()
        todo!()
    }
}
*/

fn build_single_row_write<R: Table>(row: R) -> Result<Write, ConvertError> {
    let mut wb = WriteBuilder::with_row_capacity(1);
    wb.add_row(row)?;
    Ok(wb.into_proto())
}

fn build_many_row_write<I, R>(rows: I) -> Result<Write, ConvertError>
where
    I: IntoIterator<Item = R>,
    R: Table,
{
    let iter = rows.into_iter();
    let (low, high) = iter.size_hint();

    let mut dst = WriteBuilder::with_row_capacity(high.unwrap_or(low));
    dst.add_rows(iter)?;
    Ok(dst.into_proto())
}

impl Transaction<'_, '_> {
    pub fn delete<R: Table>(&mut self, key_set: &mut KeySet<R>) -> &mut Self {
        self.add_mutation(Operation::Delete(Delete {
            table: R::NAME.to_owned(),
            key_set: Some(key_set.to_proto()),
        }))
    }

    pub fn replace<R>(&mut self, row: R) -> Result<&mut Self, ConvertError>
    where
        R: Table,
    {
        Ok(self.add_mutation(Operation::Replace(build_single_row_write(row)?)))
    }

    pub const fn state(&self) -> Option<TxState> {
        if self.completed_successfully {
            self.state
        } else {
            None
        }
    }

    pub fn replace_many<I, R>(&mut self, rows: I) -> Result<&mut Self, ConvertError>
    where
        I: IntoIterator<Item = R>,
        R: Table,
    {
        Ok(self.add_mutation(Operation::Replace(build_many_row_write(rows)?)))
    }

    pub fn update<R>(&mut self, row: R) -> Result<&mut Self, ConvertError>
    where
        R: Table,
    {
        Ok(self.add_mutation(Operation::Update(build_single_row_write(row)?)))
    }

    pub fn update_many<I, R>(&mut self, rows: I) -> Result<&mut Self, ConvertError>
    where
        I: IntoIterator<Item = R>,
        R: Table,
    {
        Ok(self.add_mutation(Operation::Update(build_many_row_write(rows)?)))
    }

    pub fn insert_or_update<R>(&mut self, row: R) -> Result<&mut Self, ConvertError>
    where
        R: Table,
    {
        Ok(self.add_mutation(Operation::InsertOrUpdate(build_single_row_write(row)?)))
    }

    pub fn insert_or_update_many<I, R>(&mut self, rows: I) -> Result<&mut Self, ConvertError>
    where
        I: IntoIterator<Item = R>,
        R: Table,
    {
        Ok(self.add_mutation(Operation::InsertOrUpdate(build_many_row_write(rows)?)))
    }

    pub fn insert<R>(&mut self, row: R) -> Result<&mut Self, ConvertError>
    where
        R: Table,
    {
        Ok(self.add_mutation(Operation::Insert(build_single_row_write(row)?)))
    }

    pub fn insert_many<I, R>(&mut self, rows: I) -> Result<&mut Self, ConvertError>
    where
        I: IntoIterator<Item = R>,
        R: Table,
    {
        Ok(self.add_mutation(Operation::Insert(build_many_row_write(rows)?)))
    }

    fn add_mutation(&mut self, operation: Operation) -> &mut Self {
        self.mutations.push(Mutation {
            operation: Some(operation),
        });
        self
    }

    pub async fn commit(&mut self) -> crate::Result<CommitResponse> {
        // make sure we're not trying to commit a transaction that failed to roll back.
        debug_assert!(matches!(self.state, None | Some(TxState::Committed)));

        self.state = Some(TxState::Committed);

        // try once before assembling types to handle backoff
        if let Some(resp) = self.try_commit_once().await? {
            return Ok(resp);
        }

        let mut backoff = Backoff::default();

        loop {
            match backoff.backoff_once() {
                Some(backoff) => backoff.await,
                None => return Err(crate::Error::TransactionContention),
            }

            if let Some(resp) = self.try_commit_once().await? {
                return Ok(resp);
            }
        }
    }

    async fn try_commit_once(&mut self) -> crate::Result<Option<CommitResponse>> {
        /*
        debug_assert!(
            self.completed_successfully,
            "comitting an already comitted stream?"
        );
        */

        let mutations = self.mutations.clone();

        let result = self.parts().commit_inner(mutations).await;

        match result {
            Ok(resp) => {
                println!("TX: comitted {} rows", self.mutations.len());
                self.mutations.clear();
                self.completed_successfully = true;
                Ok(Some(resp.into_inner()))
            }
            Err(crate::Error::Status(err)) if err.code() == Code::Aborted => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn rollback(mut self) -> crate::Result<()> {
        debug_assert!(
            !self.completed_successfully,
            "can't roll back an already rolled back transaction"
        );
        self.state = Some(TxState::RolledBack);

        self.parts().rollback().await?;

        self.completed_successfully = true;
        Ok(())
    }
}
