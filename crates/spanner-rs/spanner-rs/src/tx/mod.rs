mod snapshot;
mod transaction;

use std::marker::PhantomData;

use bytes::Bytes;
use protos::spanner::transaction_options::read_only::TimestampBound;
use protos::spanner::transaction_options::read_write::ReadLockMode;
use protos::spanner::transaction_options::{self, Mode};
use protos::spanner::transaction_selector::Selector;
use protos::spanner::{self, commit_request, TransactionOptions, TransactionSelector};
pub use transaction::{ShouldCommit, Transaction};

pub(crate) const READ_WRITE: TransactionOptions = TransactionOptions {
    exclude_txn_from_change_streams: false,
    mode: Some(Mode::ReadWrite(transaction_options::ReadWrite {
        multiplexed_session_previous_transaction_id: Bytes::new(),
        read_lock_mode: ReadLockMode::Pessimistic as i32,
    })),
};

pub(crate) const READ_ONLY: TransactionOptions = TransactionOptions {
    exclude_txn_from_change_streams: false,
    mode: Some(Mode::ReadOnly(transaction_options::ReadOnly {
        return_read_timestamp: true,
        timestamp_bound: Some(TimestampBound::Strong(true)),
    })),
};

pub trait ReadOnlyTx: crate::private::SealedTx {
    /// Build a raw proto transaction selector from self for a __read only__ transaction.
    fn build_read_only_selector(&self) -> TransactionSelector;
}

pub trait ReadWriteTx: ReadOnlyTx {
    /// Build a raw proto transaction selector from self for a __read-write__ transaction,
    /// used in the CommitReqeust message.
    fn build_read_write(&self) -> commit_request::Transaction;
}

pub trait TxOptions: crate::private::SealedTx {
    const OPTIONS: TransactionOptions;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadOnly;

impl crate::private::SealedTx for ReadOnly {}

impl TxOptions for ReadOnly {
    const OPTIONS: TransactionOptions = READ_ONLY;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadWrite;

impl crate::private::SealedTx for ReadWrite {}

impl TxOptions for ReadWrite {
    const OPTIONS: TransactionOptions = READ_WRITE;
}

/// Describes a single use transaction, that's used/created
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SingleUse;

impl crate::private::SealedTx for SingleUse {}

impl ReadOnlyTx for SingleUse {
    #[inline]
    fn build_read_only_selector(&self) -> TransactionSelector {
        TransactionSelector {
            selector: Some(Selector::SingleUse(READ_ONLY)),
        }
    }
}
impl ReadWriteTx for SingleUse {
    #[inline]
    fn build_read_write(&self) -> commit_request::Transaction {
        commit_request::Transaction::SingleUseTransaction(READ_WRITE)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Existing<'a, T: Copy> {
    tx: &'a spanner::Transaction,
    _tx_type_marker: PhantomData<T>,
}

impl<'a, T: Copy> Existing<'a, T> {
    pub(crate) const fn new(tx: &'a spanner::Transaction) -> Self {
        Self {
            tx,
            _tx_type_marker: PhantomData,
        }
    }
}

impl<T: Copy> Existing<'_, T> {
    #[inline]
    pub(crate) fn clone_id(&self) -> Bytes {
        self.tx.id.clone()
    }
}

impl<T: Copy> crate::private::SealedTx for Existing<'_, T> {}

impl ReadOnlyTx for Existing<'_, ReadOnly> {
    #[inline]
    fn build_read_only_selector(&self) -> TransactionSelector {
        TransactionSelector {
            selector: Some(Selector::Id(self.clone_id())),
        }
    }
}

impl ReadOnlyTx for Existing<'_, ReadWrite> {
    #[inline]
    fn build_read_only_selector(&self) -> TransactionSelector {
        TransactionSelector {
            selector: Some(Selector::Id(self.clone_id())),
        }
    }
}

impl ReadWriteTx for Existing<'_, ReadWrite> {
    #[inline]
    fn build_read_write(&self) -> commit_request::Transaction {
        commit_request::Transaction::TransactionId(self.clone_id())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Begin<T: Copy>(PhantomData<T>);

impl Begin<ReadWrite> {
    pub(crate) const READ_WRITE: Self = Self(PhantomData);
}

impl Begin<ReadOnly> {
    pub(crate) const READ_ONLY: Self = Self(PhantomData);
}

impl<T: TxOptions> Begin<T> {
    #[inline]
    pub const fn build_selector(&self) -> TransactionSelector {
        TransactionSelector {
            selector: Some(Selector::Begin(T::OPTIONS)),
        }
    }
}

impl<T: TxOptions> crate::private::SealedTx for Begin<T> {}

impl<T: TxOptions> TxOptions for Begin<T> {
    const OPTIONS: TransactionOptions = T::OPTIONS;
}
