use std::fmt;
use std::sync::Arc;

use protos::firestore::Write;
use protos::firestore::document_transform::FieldTransform;
use protos::firestore::document_transform::field_transform::TransformType;
use protos::firestore::write::Operation;

use crate::Error;
use crate::client::FirestoreClient;
use crate::ser::{escape_field_path, serialize_doc_fields};

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

    ///
    pub async fn commit(self) -> crate::Result<()> {
        let Self {
            writes,
            client,
            qualified_db_path,
        } = self;

        if writes.is_empty() {
            return Ok(());
        }

        let request = protos::firestore::BatchWriteRequest {
            // we need to clone the string, not the arc, so we explicitely call deref first.
            database: qualified_db_path.to_string(),
            writes,
            labels: Default::default(),
        };

        let resp = client.get().batch_write(request).await?.into_inner();
        Error::check_many_rpc_statuses(resp.status)
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

    fn set_update_inner<N, D>(self, doc: &D, use_field_masks: bool) -> crate::Result<usize>
    where
        D: serde::Serialize,
        N: crate::ser::NullStrategy,
    {
        let serialized = serialize_doc_fields::<D, N>(doc)?;

        let doc = protos::firestore::Document {
            name: self.doc_path,
            fields: serialized.fields,
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
            update_mask: use_field_masks.then_some(serialized.field_mask),
            update_transforms: Vec::new(),
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
        self.set_update_inner::<crate::ser::NullOverwrite, D>(doc, false)
    }

    /// Updates a document as part of this batch write. Returns the encoded size of
    /// the document via [`prost::Message::encoded_len`]
    pub fn update<D>(self, doc: &D) -> crate::Result<usize>
    where
        D: serde::Serialize,
    {
        self.set_update_inner::<crate::ser::OmitNulls, D>(doc, true)
    }
}
