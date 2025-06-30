use std::fmt;
use std::sync::Arc;

use protos::firestore::document_transform::FieldTransform;
use protos::firestore::document_transform::field_transform::TransformType;
use protos::firestore::write::Operation;
use protos::firestore::{BatchWriteResponse, Write};

use crate::Error;
use crate::client::FirestoreClient;
use crate::ser::{escape_field_path, serialize_write};

#[derive(Debug)]
pub struct BatchWrite {
    client: FirestoreClient,
    qualified_db_path: Arc<str>,
    writes: Vec<Write>,
}

impl BatchWrite {
    pub(crate) fn new(client: FirestoreClient, qualified_db_path: Arc<str>) -> Self {
        Self {
            client,
            qualified_db_path,
            writes: Vec::new(),
        }
    }

    pub(crate) fn new_with_write_capacity(
        client: FirestoreClient,
        qualified_db_path: Arc<str>,
        capacity: usize,
    ) -> Self {
        Self {
            client,
            qualified_db_path,
            writes: Vec::with_capacity(capacity),
        }
    }

    /// returns the number of writes that will be committed when [`commit`] is called.
    pub fn len(&self) -> usize {
        self.writes.len()
    }

    /// returns true if there are no queued writes.
    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    pub async fn commit_raw(self) -> crate::Result<BatchWriteResponse> {
        let Self {
            writes,
            client,
            qualified_db_path,
        } = self;

        if writes.is_empty() {
            return Ok(BatchWriteResponse::default());
        }

        let request = protos::firestore::BatchWriteRequest {
            database: qualified_db_path.as_ref().to_owned(),
            writes,
            labels: Default::default(),
        };

        let resp = client.get().batch_write(request).await?.into_inner();
        Ok(resp)
    }

    pub async fn commit_inspect(
        self,
    ) -> crate::Result<impl ExactSizeIterator<Item = WriteResult> + Send + Sync + 'static> {
        let count = self.writes.len();
        let BatchWriteResponse {
            write_results,
            status,
        } = self.commit_raw().await?;

        debug_assert_eq!(write_results.len(), status.len());
        debug_assert_eq!(write_results.len(), count);

        Ok(write_results
            .into_iter()
            .zip(status)
            .map(
                |(write_result, status)| match Error::check_rpc_status(status) {
                    Ok(()) => WriteResult::Succeeded(write_result),
                    Err(error) => WriteResult::Error(error),
                },
            ))
    }

    ///
    pub async fn commit(self) -> crate::Result<()> {
        let raw = self.commit_raw().await?;
        Error::check_many_rpc_statuses(raw.status)
    }

    pub fn collection<C>(&mut self, collection_name: C) -> BatchWriteCollectionRef<'_>
    where
        C: fmt::Display,
    {
        BatchWriteCollectionRef {
            parent_path: format!("{}/documents", &self.qualified_db_path),
            collection_name: collection_name.to_string(),
            writes: &mut self.writes,
        }
    }
}

#[derive(Debug)]
pub enum WriteResult {
    Succeeded(protos::firestore::WriteResult),
    Error(crate::Error),
}

pub struct BatchWriteCollectionRef<'a> {
    parent_path: String,
    collection_name: String,
    writes: &'a mut Vec<Write>,
}

impl<'a> BatchWriteCollectionRef<'a> {
    pub fn doc<D>(&mut self, doc_id: D) -> BatchDocRef<'_>
    where
        D: fmt::Display,
    {
        BatchDocRef {
            doc_path: format!("{}/{}/{}", self.parent_path, self.collection_name, doc_id),
            writes: self.writes,
        }
    }
}

pub struct BatchDocRef<'a> {
    doc_path: String,
    writes: &'a mut Vec<Write>,
}

impl<'a> BatchDocRef<'a> {
    pub fn collection<C>(&mut self, collection_name: C) -> BatchWriteCollectionRef<'_>
    where
        C: fmt::Display,
    {
        BatchWriteCollectionRef {
            parent_path: self.doc_path.clone(),
            collection_name: collection_name.to_string(),
            writes: self.writes,
        }
    }

    pub fn delete(self) {
        self.writes.push(Write {
            update_mask: None,
            update_transforms: Vec::new(),
            current_document: None,
            operation: Some(Operation::Delete(self.doc_path)),
        });
    }

    /// If the document doesn't exist, this will return an error indicating no delete was performed.
    pub fn delete_existing(self) {
        self.writes.push(Write {
            update_mask: None,
            update_transforms: Vec::new(),
            current_document: Some(protos::firestore::Precondition {
                condition_type: Some(protos::firestore::precondition::ConditionType::Exists(true)),
            }),
            operation: Some(Operation::Delete(self.doc_path)),
        });
    }

    pub fn field_transforms<I, F>(self, transforms: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = (F, TransformType)>,
        F: AsRef<str>,
    {
        let field_transforms = transforms
            .into_iter()
            .map(|(field, transform)| FieldTransform {
                field_path: escape_field_path(field.as_ref()),
                transform_type: Some(transform),
            })
            .collect::<Vec<_>>();

        if field_transforms.is_empty() {
            return Ok(());
        }

        self.writes.push(Write {
            update_mask: None,
            update_transforms: Vec::new(),
            current_document: None,
            operation: Some(Operation::Transform(protos::firestore::DocumentTransform {
                document: self.doc_path,
                field_transforms,
            })),
        });

        Ok(())
    }

    fn set_update_inner<W, D>(self, doc: &D, use_field_masks: bool) -> crate::Result<usize>
    where
        D: serde::Serialize,
        W: crate::ser::WriteKind,
    {
        let (doc, update_transforms) = serialize_write::<W>(doc)?;

        let (fields, update_mask) = if use_field_masks {
            (doc.into_fields(), None)
        } else {
            let (doc, mask) = doc.into_fields_with_mask();
            (doc, Some(mask))
        };

        let doc = protos::firestore::Document {
            name: self.doc_path,
            fields,
            create_time: None,
            update_time: None,
        };

        let size = crate::util::encoded_document_size(&doc);

        if size > crate::doc::MAX_DOCUMENT_SIZE {
            return Err(Error::OverSizeLimit {
                document_id: doc.name,
                size,
            });
        }

        self.writes.push(Write {
            update_mask,
            update_transforms,
            current_document: None,
            operation: Some(Operation::Update(doc)),
        });

        Ok(size)
    }

    /// Sets a document as part of this batch write. Returns the encoded size of the
    /// document via [`prost::Message::encoded_len`]
    pub fn set<D>(self, doc: &D) -> crate::Result<usize>
    where
        D: serde::Serialize,
    {
        self.set_update_inner::<crate::ser::Update, D>(doc, false)
    }

    /// Updates a document as part of this batch write. Returns the encoded size of
    /// the document via [`prost::Message::encoded_len`]
    pub fn update<D>(self, doc: &D) -> crate::Result<usize>
    where
        D: serde::Serialize,
    {
        self.set_update_inner::<crate::ser::Merge, D>(doc, true)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_batch_write_and_delete() -> crate::Result<()> {
        let client =
            crate::Firestore::new("mysticetus-oncloud", gcp_auth_channel::Scope::Firestore).await?;

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Data {
            field: String,
        }

        fn new_doc(field: impl ToString) -> Data {
            Data {
                field: field.to_string(),
            }
        }

        client
            .collection("test-docs")
            .doc("existing-doc")
            .set(&new_doc("existing doc"))
            .await?;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let mut write = client.batch_write();

        let mut collec = write.collection("test-docs");

        collec.doc("new-doc").set(&new_doc("new_doc"))?;
        collec.doc("existing-doc").delete_existing();
        collec.doc("not-existing-doc").delete_existing();

        let mut resp = write.commit_inspect().await?;
        let doc_add = resp.next().unwrap();
        let doc_delete = resp.next().unwrap();
        let doc_delete_missing = resp.next().unwrap();

        match dbg!(doc_add) {
            crate::batch::write::WriteResult::Succeeded(write) => {
                assert!(write.update_time.is_some())
            }
            _ => panic!("first write should have succeeded"),
        }

        match dbg!(doc_delete) {
            crate::batch::write::WriteResult::Succeeded(write) => {
                assert!(write.update_time.is_none())
            }
            _ => panic!("first delete should have succeeded"),
        }

        match dbg!(doc_delete_missing) {
            crate::batch::write::WriteResult::Error(error) => {
                assert_eq!(error.rpc_code(), Some(tonic::Code::NotFound))
            }
            _ => panic!("missing doc should have returned an error"),
        }

        assert!(resp.next().is_none());

        Ok(())
    }
}
