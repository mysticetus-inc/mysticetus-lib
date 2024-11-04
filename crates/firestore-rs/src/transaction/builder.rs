use protos::firestore::TransactionOptions;
use protos::firestore::transaction_options::{Mode, ReadOnly, read_only};

use super::Transaction;
use crate::Firestore;

pub struct TransactionBuilder {
    client: Firestore,
}

impl TransactionBuilder {
    pub(crate) fn new(client: Firestore) -> Self {
        Self { client }
    }

    pub async fn read_only(self) -> crate::Result<Transaction<Firestore>> {
        let opts = TransactionOptions {
            mode: Some(Mode::ReadOnly(ReadOnly {
                consistency_selector: None,
            })),
        };

        Transaction::start(self.client, Some(opts)).await
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

        Transaction::start(self.client, Some(opts)).await
    }

    pub async fn read_write(self) -> crate::Result<Transaction<Firestore>> {
        Transaction::start(self.client, None).await
    }
}
