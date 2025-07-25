#![allow(dead_code)] // while in dev

use bytes::Bytes;
use protos::firestore::{self, BeginTransactionRequest, TransactionOptions};

use crate::{CollectionRef, Firestore, PathComponent};

pub mod batch;
pub mod builder;

pub struct TransactionToken(Vec<u8>);

#[must_use = "Either [`Transaction::commit`] or [`Transaction::rollback`] must be called"]
pub struct Transaction<Ref> {
    bytes: Bytes,
    refer: Ref,
    writes: Vec<firestore::Write>,
}

impl<Ref> Transaction<Ref> {
    pub(crate) fn existing(refer: Ref, bytes: Bytes) -> Self {
        Self {
            bytes,
            refer,
            writes: Vec::new(),
        }
    }

    #[inline]
    fn into_bytes(self) -> Bytes {
        self.bytes
    }
}

impl Transaction<Firestore> {
    pub(crate) async fn start(
        parent: Firestore,
        options: TransactionOptions,
    ) -> crate::Result<Self> {
        let req = BeginTransactionRequest {
            database: parent.qualified_db_path().to_owned(),
            options: Some(options),
        };

        let resp = parent
            .client
            .get()
            .begin_transaction(req)
            .await?
            .into_inner();

        Ok(Self {
            refer: parent,
            bytes: resp.transaction,
            writes: Vec::new(),
        })
    }

    pub fn collection<C>(&self, collection_name: C) -> TransactionCollectionRef<'_, C>
    where
        C: PathComponent,
    {
        TransactionCollectionRef {
            collec_ref: self.refer.collection(collection_name),
            bytes: &self.bytes,
        }
    }

    pub async fn rollback(self) -> crate::Result<()> {
        let req = firestore::RollbackRequest {
            database: self.refer.qualified_db_path().to_owned(),
            transaction: self.bytes,
        };

        self.refer.client.get().rollback(req).await?;

        Ok(())
    }

    pub async fn commit(self) -> crate::Result<Vec<firestore::WriteResult>> {
        let req = firestore::CommitRequest {
            database: self.refer.qualified_db_path().to_owned(),
            writes: self.writes,
            transaction: self.bytes,
        };

        let resp = self.refer.client.get().commit(req).await?.into_inner();

        Ok(resp.write_results)
    }
}

pub struct TransactionCollectionRef<'a, C: PathComponent> {
    bytes: &'a Bytes,
    collec_ref: CollectionRef<C>,
}
