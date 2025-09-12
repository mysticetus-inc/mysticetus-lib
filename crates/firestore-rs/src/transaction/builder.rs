use bytes::Bytes;
use protos::firestore::TransactionOptions;
use protos::firestore::transaction_options::{Mode, ReadOnly, read_only};

use super::Transaction;
use crate::Firestore;

pub struct TransactionBuilder {
    client: Firestore,
}

const READ_WRITE_OPTIONS: TransactionOptions = TransactionOptions {
    mode: Some(Mode::ReadWrite(
        protos::firestore::transaction_options::ReadWrite {
            retry_transaction: Bytes::new(),
        },
    )),
};

const DEFAULT_READ_ONLY_OPTIONS: TransactionOptions = TransactionOptions {
    mode: Some(Mode::ReadOnly(ReadOnly {
        consistency_selector: None,
    })),
};

impl TransactionBuilder {
    pub(crate) fn new(client: Firestore) -> Self {
        Self { client }
    }

    pub async fn read_only(self) -> crate::Result<Transaction<Firestore>> {
        Transaction::start(self.client, DEFAULT_READ_ONLY_OPTIONS).await
    }

    pub async fn read_only_at(
        self,
        time: timestamp::Timestamp,
    ) -> crate::Result<Transaction<Firestore>> {
        let opts = TransactionOptions {
            mode: Some(Mode::ReadOnly(ReadOnly {
                consistency_selector: Some(read_only::ConsistencySelector::ReadTime(time.into())),
            })),
        };

        Transaction::start(self.client, opts).await
    }

    pub async fn read_write(self) -> crate::Result<Transaction<Firestore>> {
        Transaction::start(self.client, READ_WRITE_OPTIONS).await
    }
}
