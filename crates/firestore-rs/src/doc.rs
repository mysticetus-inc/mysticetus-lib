//! Contains types to work with Firestore documents.

use std::borrow::Cow;
use std::collections::HashMap;

use futures::Stream;
use protos::firestore::document_transform::FieldTransform;
use protos::firestore::document_transform::field_transform::{ServerValue, TransformType};
use protos::firestore::get_document_request::ConsistencySelector;
use protos::firestore::precondition::ConditionType;
use protos::firestore::transaction_options::{Mode, ReadWrite};
use protos::firestore::{
    self, BeginTransactionRequest, CommitRequest, CommitResponse, DeleteDocumentRequest, Document,
    DocumentMask, GetDocumentRequest, Precondition, RollbackRequest, TransactionOptions,
    UpdateDocumentRequest, Write,
};
use timestamp::Timestamp;

/// The firestore document size limit. 1MiB - 4 bytes;
pub const MAX_DOCUMENT_SIZE: usize = 1_048_576;

use crate::batch::read;
use crate::client::ResponseExt;
use crate::collec::CollectionRef;
use crate::de::deserialize_doc_fields;
use crate::ser::DocFields;
use crate::{Error, PathComponent, Reference, ser};

/// Deserialized document from Firestore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc<O> {
    doc_fields: O,
    pub reference: Box<Reference>,
    pub create_time: Option<Timestamp>,
    pub update_time: Option<Timestamp>,
}

impl<O> Doc<O> {
    #[inline]
    pub fn id(&self) -> &str {
        self.reference
            .as_str()
            .rsplit_once('/')
            .map(|(_leading, id)| id)
            .unwrap_or(self.reference.as_str())
    }

    #[inline]
    pub fn into_inner(self) -> O {
        self.doc_fields
    }

    #[inline]
    pub fn fields(&self) -> &O {
        &self.doc_fields
    }

    #[inline]
    pub fn fields_mut(&mut self) -> &mut O {
        &mut self.doc_fields
    }
}

impl RawDoc {
    pub fn deserialize_fields<'de, O>(self) -> crate::Result<Doc<O>>
    where
        O: serde::Deserialize<'de>,
    {
        let doc_fields = deserialize_doc_fields(self.doc_fields)?;

        Ok(Doc {
            reference: self.reference,
            doc_fields,
            create_time: self.create_time,
            update_time: self.update_time,
        })
    }
}

impl<O> Doc<O> {
    pub(crate) fn from_document<'de>(document: Document) -> crate::Result<Self>
    where
        O: serde::Deserialize<'de>,
    {
        Ok(Self {
            reference: Reference::new_string(document.name),
            doc_fields: deserialize_doc_fields(document.fields)?,
            create_time: document.create_time.map(|ts| ts.into()),
            update_time: document.update_time.map(|ts| ts.into()),
        })
    }
}

/// A raw document, with the inner contents beting the unchanged proto fields.
pub type RawDoc = Doc<HashMap<String, firestore::Value>>;

// Raw document conversion to avoid re-serializing into a generic type.
impl From<Document> for RawDoc {
    fn from(document: Document) -> Self {
        Self {
            reference: Reference::new_string(document.name),
            doc_fields: document.fields,
            create_time: document.create_time.map(|ts| ts.into()),
            update_time: document.update_time.map(|ts| ts.into()),
        }
    }
}

/// A reference to a firestore document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRef<C: PathComponent, D: PathComponent> {
    doc_id: D,
    reference: Box<Reference>,
    pub(super) collec_ref: CollectionRef<C>,
}

impl<C: PathComponent, D: PathComponent> DocumentRef<C, D> {
    pub(crate) fn new(collec_ref: CollectionRef<C>, doc_id: D) -> Self {
        let mut qualified_path = collec_ref.qualified_path().to_owned();

        doc_id.append_to_path(&mut qualified_path);

        Self {
            doc_id,
            collec_ref,
            reference: Reference::new_string(qualified_path),
        }
    }

    pub fn id(&self) -> &D {
        &self.doc_id
    }

    pub fn reference(&self) -> &crate::Reference {
        &self.reference
    }

    /// Gets a subcollection underneath this document.
    pub fn collection<S: PathComponent>(self, collection: S) -> CollectionRef<S> {
        CollectionRef::new_nested(collection, self.reference, self.collec_ref.parent)
    }

    pub async fn update_transaction_raw<F, T>(
        &mut self,
        transaction_fn: F,
    ) -> crate::Result<Option<CommitResponse>>
    where
        F: FnOnce(Option<RawDoc>) -> Option<T>,
        T: serde::Serialize,
    {
        let req = BeginTransactionRequest {
            database: self.collec_ref.parent.qualified_db_path().to_owned(),
            options: Some(TransactionOptions {
                mode: Some(Mode::ReadWrite(ReadWrite {
                    retry_transaction: bytes::Bytes::new(),
                })),
            }),
        };

        let transaction_bytes = self
            .collec_ref
            .parent
            .client
            .get()
            .begin_transaction(req)
            .await?
            .into_inner()
            .transaction;

        let consistency = Some(ConsistencySelector::Transaction(transaction_bytes.clone()));

        let document = self.get_doc_inner(consistency, None).await?;

        let init_doc = document.map(RawDoc::from_document).transpose()?;

        match transaction_fn(init_doc) {
            Some(update) => {
                let DocFields { fields, field_mask } =
                    ser::serialize_doc_fields::<T, ser::NullOverwrite>(&update)?;

                let document = Document {
                    fields,
                    name: self.reference.to_string(),
                    create_time: None,
                    update_time: None,
                };

                let commit_req = CommitRequest {
                    database: self.collec_ref.parent.qualified_db_path().to_owned(),
                    writes: vec![Write {
                        current_document: None,
                        update_mask: Some(field_mask),
                        update_transforms: vec![],
                        operation: Some(protos::firestore::write::Operation::Update(document)),
                    }],
                    transaction: transaction_bytes,
                };

                let commit_resp = self
                    .collec_ref
                    .parent
                    .client
                    .get()
                    .commit(commit_req)
                    .await?
                    .into_inner();

                Ok(Some(commit_resp))
            }
            None => {
                let rollback_req = RollbackRequest {
                    database: self.collec_ref.parent.qualified_db_path().to_owned(),
                    transaction: transaction_bytes,
                };

                self.collec_ref
                    .parent
                    .client
                    .get()
                    .rollback(rollback_req)
                    .await?;
                Ok(None)
            }
        }
    }

    pub fn parent(&self) -> &CollectionRef<C> {
        &self.collec_ref
    }

    pub fn list_collection_ids(&mut self) -> impl Stream<Item = crate::Result<Vec<String>>> + '_ {
        crate::common::list_collection_ids(
            &mut self.collec_ref.parent.client,
            self.reference.as_str(),
        )
    }

    pub fn subcollection_batch_read(&self) -> read::BatchRead<read::Document> {
        read::BatchRead::new(
            self.collec_ref.parent.client.clone(),
            self.reference.to_string().into(),
            None,
        )
    }

    async fn write(&mut self, write: Write) -> crate::Result<Option<firestore::WriteResult>> {
        let database = crate::try_extract_database_path(self.reference.as_str())
            .expect("document has an invalid ID")
            .to_owned();

        let request = firestore::BatchWriteRequest {
            writes: vec![write],
            database,
            labels: Default::default(),
        };

        let mut resp = self
            .collec_ref
            .parent
            .client
            .get()
            .batch_write(request)
            .await?
            .into_inner();

        let write_result = match resp.write_results.len() {
            0 => None,
            1 => resp.write_results.pop(),
            _ => {
                warn!("received multiple write_results for a single write? {resp:#?}");
                Some(resp.write_results.swap_remove(0))
            }
        };

        let status = match resp.status.len() {
            0 => return Ok(write_result),
            1 => resp.status.pop().unwrap(),
            _ => {
                warn!("received multiple status results for a single write? {resp:#?}");
                resp.status.swap_remove(0)
            }
        };

        crate::Error::check_rpc_status(status).map(|_| write_result)
    }

    /*
    pub async fn transaction<F, O, R>(&self, transaction_fn: F) -> Result<Option<Doc<O>>, TransactionError>
    where
        F: FnOnce(Option<&mut O>) -> R,
        R: Into<TransactionResult>,
        O: serde::Serialize + serde::de::DeserializeOwned + Clone
    {
        // todo: Keep this around in [`DocumentRef`], that way we dont need to do this every time
        // to get the qualified database path
        let database = self.qualified_doc_path.split('/')
            .take(4)
            .collect::<Vec<&str>>()
            .join("/");

        let transaction_req = BeginTransactionRequest {
            database: database.clone(),
            options: None,
        };

        let mut client = self.auth_channel.build_client().await?;

        let transaction_resp = client.begin_transaction(transaction_req).await?.into_inner();

        let orig_doc = self.get().await?;
        // Hold onto a clone so we can return the original if the transaction is aborted.
        let mut transaction_doc = orig_doc.clone();

        let doc_mut = transaction_doc.as_mut().map(|doc| doc.fields_mut());

        match transaction_fn(doc_mut).into() {
            TransactionResult::Commit => (),
            TransactionResult::Abort => return Ok(orig_doc),
            TransactionResult::Error(err) => return Err(TransactionError::Transaction(err)),
        }

        let operation = match transaction_doc.as_ref() {
            Some(doc) => {
                let document = Document {
                    fields: serialize_doc_fields::<_, false>(doc.fields())?.fields,
                    name: self.qualified_doc_path.clone(),
                    update_time: None,
                    create_time: None,
                };

                Operation::Update(document)
            },
            None => Operation::Delete(self.qualified_doc_path.clone()),
        };

        let write = firestore::Write {
            operation: Some(operation),
            update_mask: None,
            update_transforms: vec![],
            current_document: None,
        };

        let commit_request = CommitRequest {
            database,
            writes: vec![write],
            transaction: transaction_resp.transaction,
        };

        client.commit(commit_request).await?;

        Ok(transaction_doc)
    }
    */

    pub async fn get_field<S>(&mut self, field: S) -> crate::Result<Option<crate::Value>>
    where
        S: Into<String>,
    {
        let field: String = field.into();

        let req = GetDocumentRequest {
            name: self.reference.to_string(),
            consistency_selector: None,
            mask: Some(DocumentMask {
                field_paths: vec![field.clone()],
            }),
        };

        let mut doc = match self
            .collec_ref
            .parent
            .client
            .get()
            .get_document(req)
            .await
            .handle_not_found()?
        {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut component_iter = field
            .split('.')
            .map(|s| s.trim_start_matches('`').trim_end_matches('`'));

        let mut current = match component_iter
            .next()
            .and_then(|component| doc.fields.remove(component))
        {
            Some(firestore::Value {
                value_type: Some(value_type),
            }) => value_type,
            _ => return Ok(None),
        };

        for component in component_iter {
            match current {
                firestore::value::ValueType::MapValue(mut map) => {
                    match map.fields.remove(component) {
                        Some(firestore::Value {
                            value_type: Some(value_type),
                        }) => current = value_type,
                        _ => return Ok(None),
                    }
                }
                firestore::value::ValueType::ArrayValue(mut array) => {
                    let index = match component.parse::<usize>() {
                        Ok(idx) => idx,
                        Err(_) => return Ok(None),
                    };

                    if array.values.len() > index {
                        current = array.values.swap_remove(index).value_type.unwrap();
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Ok(None),
            }
        }

        Ok(Some(crate::Value::from_proto_value_type(current)))
    }

    /*
    pub async fn set_field<S>(&mut self, field: S, value: crate::Value) -> crate::Result<RawDoc>
    where
        S: AsRef<str>,
    {
        let path = field.as_ref();
        let mut fields = HashMap::with_capacity(1);
        let mut rpath_iter = path.rsplit_terminator('.');

        let mut dst = match rpath_iter.next() {
            Some(field) => {
                if field.starts_with('`') && field.ends_with('`') {
                    let trimmed = field.trim_start_matches('`').trim_end_matches('`');
                    todo!();
                }
            }
            None => panic!("cannot have an empty field path"),
        };

        let req = UpdateDocumentRequest {
            document: Some(Document {
                name: self.qualified_path.clone(),
                fields,
                update_time: None,
                create_time: None,
            }),
            update_mask: Some(DocumentMask {
                field_paths: vec![path.to_owned()],
            }),
            mask: None,
            current_document: None,
        };

        let resp = self
            .collec_ref
            .parent
            .client
            .get()
            .update_document(req)
            .await?
            .into_inner();

        RawDoc::from_document(resp)
    }
    */

    async fn get_doc_inner(
        &mut self,
        consistency_selector: Option<ConsistencySelector>,
        mask: Option<DocumentMask>,
    ) -> crate::Result<Option<Document>> {
        let request = firestore::GetDocumentRequest {
            name: self.reference.to_string(),
            mask,
            consistency_selector,
        };

        self.collec_ref
            .parent
            .client
            .get()
            .get_document(request)
            .await
            .handle_not_found()
            .map_err(crate::Error::from)
    }

    pub async fn get_raw(&mut self) -> crate::Result<Option<RawDoc>> {
        match self.get_doc_inner(None, None).await? {
            Some(inner) => Ok(Some(inner.into())),
            None => Ok(None),
        }
    }

    pub async fn get_raw_masked(
        &mut self,
        fields: impl IntoIterator<Item: AsRef<str>>,
    ) -> crate::Result<Option<RawDoc>> {
        match self.get_doc_inner(None, encode_mask(fields)).await? {
            Some(inner) => Ok(Some(inner.into())),
            None => Ok(None),
        }
    }

    pub async fn get<'de, O>(&mut self) -> crate::Result<Option<Doc<O>>>
    where
        O: serde::Deserialize<'de>,
    {
        match self.get_doc_inner(None, None).await? {
            Some(document) => Doc::from_document(document).map(Some),
            None => Ok(None),
        }
    }

    pub async fn get_masked<'de, O>(
        &mut self,
        fields: impl IntoIterator<Item: AsRef<str>>,
    ) -> crate::Result<Option<Doc<O>>>
    where
        O: serde::Deserialize<'de>,
    {
        match self.get_doc_inner(None, encode_mask(fields)).await? {
            Some(document) => Doc::from_document(document).map(Some),
            None => Ok(None),
        }
    }

    /// Deletes the document.
    pub async fn delete(&mut self) -> crate::Result<()> {
        let request = DeleteDocumentRequest {
            name: self.reference.to_string(),
            current_document: None,
        };

        self.collec_ref
            .parent
            .client
            .get()
            .delete_document(request)
            .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", name = "update_inner", skip(doc_fields))]
    async fn update_inner(
        &self,
        doc_fields: DocFields,
        precond: Option<Precondition>,
        omit_mask: bool,
    ) -> crate::Result<RawDoc> {
        let request = UpdateDocumentRequest {
            document: Some(Document {
                name: self.reference.to_string(),
                fields: doc_fields.fields,
                update_time: None,
                create_time: None,
            }),
            update_mask: if omit_mask {
                None
            } else {
                Some(doc_fields.field_mask)
            },
            mask: None,
            current_document: precond,
        };

        let resp = self
            .collec_ref
            .parent
            .client
            .get()
            .update_document(request)
            .await?;

        Ok(RawDoc::from(resp.into_inner()))
    }

    pub async fn update_with_condition<'de, T>(
        &self,
        doc: &T,
        must_already_exist: bool,
    ) -> crate::Result<Doc<T>>
    where
        T: serde::Serialize + serde::Deserialize<'de>,
    {
        let precond = Precondition {
            condition_type: Some(ConditionType::Exists(must_already_exist)),
        };

        let doc_fields = ser::serialize_update_doc(doc)?;

        self.update_inner(doc_fields, Some(precond), false)
            .await?
            .deserialize_fields()
    }

    pub async fn update_serialized<'de, T>(&self, doc_fields: DocFields) -> crate::Result<Doc<T>>
    where
        T: serde::Deserialize<'de>,
    {
        self.update_inner(doc_fields, None, false)
            .await?
            .deserialize_fields()
    }

    pub async fn update<'de, T>(&self, doc: &T) -> crate::Result<Doc<T>>
    where
        T: serde::Serialize + serde::Deserialize<'de>,
    {
        let doc_fields = ser::serialize_update_doc(doc)?;
        self.update_inner(doc_fields, None, false)
            .await?
            .deserialize_fields()
    }

    pub async fn set<'de, T>(&self, doc: &T) -> crate::Result<Doc<T>>
    where
        T: serde::Serialize + serde::Deserialize<'de>,
    {
        let doc_fields = ser::serialize_set_doc(doc)?;

        self.update_inner(doc_fields, None, true)
            .await?
            .deserialize_fields()
    }

    pub async fn set_serialized<'de, T>(&self, doc_fields: DocFields) -> crate::Result<Doc<T>>
    where
        T: serde::Deserialize<'de>,
    {
        self.update_inner(doc_fields, None, true)
            .await?
            .deserialize_fields()
    }

    pub async fn set_serialized_with_condition<'de, T>(
        &self,
        doc_fields: DocFields,
        cond: Precondition,
    ) -> crate::Result<Doc<T>>
    where
        T: serde::Deserialize<'de>,
    {
        self.update_inner(doc_fields, Some(cond), true)
            .await?
            .deserialize_fields()
    }

    pub async fn set_with_condition<T>(
        &self,
        doc: &T,
        must_already_exist: bool,
    ) -> crate::Result<Doc<T>>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let precond = Precondition {
            condition_type: Some(ConditionType::Exists(must_already_exist)),
        };

        let doc_fields = ser::serialize_set_doc(doc)?;

        self.update_inner(doc_fields, Some(precond), true)
            .await?
            .deserialize_fields()
    }

    pub fn build_write(&mut self) -> WriteBuilder<'_, C, D, ()> {
        WriteBuilder::new(self)
    }
}

fn encode_mask(fields: impl IntoIterator<Item: AsRef<str>>) -> Option<DocumentMask> {
    let field_paths = fields
        .into_iter()
        .map(|field| crate::ser::escape_field_path(field.as_ref()))
        .collect::<Vec<String>>();

    if field_paths.is_empty() {
        None
    } else {
        Some(DocumentMask { field_paths })
    }
}

pub mod write_type {
    pub(crate) use private::WriteType;

    use super::firestore::write::Operation;

    pub(super) mod private {
        pub trait WriteType {
            fn to_operation(self) -> super::Operation;
        }
    }

    macro_rules! impl_write_type {
        ($($name:ident($inner_type:ty) => $op_variant:ident),* $(,)?) => {
            $(
                #[allow(clippy::derive_partial_eq_without_eq)]
                #[derive(Debug, Clone, PartialEq)]
                pub struct $name(pub(crate) $inner_type);

                impl private::WriteType for $name {
                    fn to_operation(self) -> Operation {
                        Operation::$op_variant(self.0)
                    }
                }
            )*
        };
    }

    // TODO: handle transform-only writes. Need to be able to apply them on-top of a write,
    // and also on their own, so I think I need to take a different approach with the generics.
    // Transform(super::firestore::DocumentTransform) => Transform,
    impl_write_type! {
        Delete(String) => Delete,
        Update(super::firestore::Document) => Update,
    }
}

pub struct WriteBuilder<'a, C: PathComponent, D: PathComponent, T> {
    doc_ref: &'a mut DocumentRef<C, D>,
    /// The encoded document size, computed via <code><[`Document`] as
    /// [`prost::Message`]>::[encoded_len]</code>
    ///
    /// Initialized to [`None`], and only inserted via [`WriteBuilder::update`] or
    /// [`WriteBuilder::set`].
    ///
    /// [encoded_len]: prost::Message::encoded_len
    doc_size: Option<usize>,
    write: T,
    mask: Option<DocumentMask>,
    transforms: Option<Vec<FieldTransform>>,
    precondition: Option<firestore::Precondition>,
}

impl<'a, C: PathComponent, D: PathComponent> WriteBuilder<'a, C, D, ()> {
    fn new(doc_ref: &'a mut DocumentRef<C, D>) -> Self {
        Self {
            doc_ref,
            doc_size: None,
            write: (),
            mask: None,
            transforms: None,
            precondition: None,
        }
    }

    /// If a document was already provided via [update] or [set].
    ///
    /// [update]: [`WriteBuilder::update`]
    /// [set]: [`WriteBuilder::set`]
    pub fn document_size(&self) -> Option<usize> {
        self.doc_size
    }

    /// method for transform only writes.
    pub async fn apply_transforms(self) -> crate::Result<Option<firestore::WriteResult>> {
        let write = Write {
            update_mask: self.mask,
            update_transforms: vec![],
            current_document: self.precondition,
            operation: Some(firestore::write::Operation::Transform(
                firestore::DocumentTransform {
                    document: self.doc_ref.reference.to_string(),
                    field_transforms: self.transforms.unwrap_or_default(),
                },
            )),
        };

        self.doc_ref.write(write).await
    }

    pub fn delete(self) -> WriteBuilder<'a, C, D, write_type::Delete> {
        let delete = write_type::Delete(self.doc_ref.reference.to_string());

        WriteBuilder {
            write: delete,
            doc_size: self.doc_size,
            doc_ref: self.doc_ref,
            mask: self.mask,
            transforms: self.transforms,
            precondition: self.precondition,
        }
    }

    pub fn set<T>(self, doc: &T) -> crate::Result<WriteBuilder<'a, C, D, write_type::Update>>
    where
        T: serde::Serialize,
    {
        let doc_fields = ser::serialize_set_doc(doc)?;
        self.set_serialized(doc_fields)
    }

    pub fn set_serialized(
        self,
        doc_fields: DocFields,
    ) -> crate::Result<WriteBuilder<'a, C, D, write_type::Update>> {
        let doc = Document {
            name: self.doc_ref.reference.to_string(),
            fields: doc_fields.fields,
            create_time: None,
            update_time: None,
        };

        let size = crate::util::encoded_document_size(&doc);

        if size > MAX_DOCUMENT_SIZE {
            return Err(Error::OverSizeLimit {
                document_id: doc.name,
                size,
            });
        }

        let write = write_type::Update(doc);

        Ok(WriteBuilder {
            write,
            doc_ref: self.doc_ref,
            doc_size: Some(size),
            mask: self.mask,
            transforms: self.transforms,
            precondition: self.precondition,
        })
    }

    pub fn update<T>(self, doc: &T) -> crate::Result<WriteBuilder<'a, C, D, write_type::Update>>
    where
        T: serde::Serialize,
    {
        let doc_fields = ser::serialize_update_doc(doc)?;
        self.update_serialized(doc_fields)
    }

    pub fn update_serialized(
        mut self,
        doc_fields: DocFields,
    ) -> crate::Result<WriteBuilder<'a, C, D, write_type::Update>> {
        if !doc_fields.field_mask.field_paths.is_empty() {
            match self.mask {
                Some(ref mut mask) => mask.field_paths.extend(doc_fields.field_mask.field_paths),
                None => {
                    self.mask = Some(DocumentMask {
                        field_paths: doc_fields.field_mask.field_paths,
                    })
                }
            }
        }

        let doc = Document {
            name: self.doc_ref.reference.to_string(),
            fields: doc_fields.fields,
            create_time: None,
            update_time: None,
        };

        let size = crate::util::encoded_document_size(&doc);

        if size > MAX_DOCUMENT_SIZE {
            return Err(Error::OverSizeLimit {
                document_id: doc.name,
                size,
            });
        }

        let write = write_type::Update(doc);

        Ok(WriteBuilder {
            write,
            doc_size: Some(size),
            doc_ref: self.doc_ref,
            mask: self.mask,
            transforms: self.transforms,
            precondition: self.precondition,
        })
    }
}

impl<'a, C: PathComponent, D: PathComponent, T> WriteBuilder<'a, C, D, T> {
    pub fn update_mask<A, S>(mut self, iter: A) -> Self
    where
        A: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let iter = iter.into_iter();
        let (low, high) = iter.size_hint();

        let mut field_paths = Vec::with_capacity(high.unwrap_or(low));

        for field_path in iter {
            if field_path.as_ref().is_empty() {
                continue;
            }

            field_paths.push(ser::escape_field_path(field_path.as_ref()));
        }

        if !field_paths.is_empty() {
            match self.mask {
                Some(ref mut mask) => mask.field_paths.extend(field_paths),
                None => self.mask = Some(DocumentMask { field_paths }),
            }
        }

        self
    }

    pub fn must_already_exist(self) -> Self {
        self.precondition(ConditionType::Exists(true))
    }

    pub fn update_time(self, timestamp: Timestamp) -> Self {
        self.precondition(ConditionType::UpdateTime(timestamp.into()))
    }

    pub fn must_not_exist(self) -> Self {
        self.precondition(ConditionType::Exists(false))
    }

    pub fn precondition(mut self, condition_type: ConditionType) -> Self {
        self.precondition = Some(Precondition {
            condition_type: Some(condition_type),
        });

        self
    }

    pub fn set_field_to_server_timestamp<S>(self, field: S) -> Self
    where
        S: Into<String>,
    {
        self.add_field_transform(field.into(), |builder| builder.set_to_server_time())
    }

    pub fn array_difference<S, V>(self, field: S, values: V) -> Self
    where
        S: Into<String>,
        V: IntoIterator,
        FirestorePrimitive: From<V::Item>,
    {
        self.add_field_transform(field.into(), |trans| {
            trans.array_difference(values.into_iter())
        })
    }
    pub fn array_union<S, V>(self, field: S, values: V) -> Self
    where
        S: Into<String>,
        V: IntoIterator,
        FirestorePrimitive: From<V::Item>,
    {
        self.add_field_transform(field.into(), |trans| trans.array_union(values.into_iter()))
    }

    pub fn field_increment<S, V>(self, field: S, value: V) -> Self
    where
        S: Into<String>,
        FirestoreNumber: From<V>,
    {
        self.add_field_transform(field.into(), |builder| builder.increment(value))
    }

    pub fn field_maximum<S, V>(self, field: S, value: V) -> Self
    where
        S: Into<String>,
        FirestoreNumber: From<V>,
    {
        self.add_field_transform(field.into(), |builder| builder.maximum(value))
    }

    pub fn field_minimum<S, V>(self, field: S, value: V) -> Self
    where
        S: Into<String>,
        FirestoreNumber: From<V>,
    {
        self.add_field_transform(field.into(), |builder| builder.minimum(value))
    }

    pub fn add_field_transform<F>(mut self, field: String, transform: F) -> Self
    where
        F: FnOnce(FieldTransformBuilder) -> FieldTransform,
    {
        let transform = transform(FieldTransformBuilder::field(field));

        self.transforms
            .get_or_insert_with(|| Vec::with_capacity(1))
            .push(transform);

        self
    }
}

impl<'a, C: PathComponent, D: PathComponent, T> WriteBuilder<'a, C, D, T>
where
    T: write_type::private::WriteType,
{
    pub(crate) fn into_parts(self) -> (&'a mut DocumentRef<C, D>, Write) {
        let write = Write {
            update_mask: self.mask,
            update_transforms: self.transforms.unwrap_or_default(),
            current_document: self.precondition,
            operation: Some(self.write.to_operation()),
        };

        (self.doc_ref, write)
    }

    pub async fn commit(self) -> crate::Result<Option<firestore::WriteResult>> {
        let (doc_ref, write) = self.into_parts();
        doc_ref.write(write).await
    }
}

pub struct FieldTransformBuilder {
    field: String,
}

pub enum FirestoreNumber {
    Float(f64),
    Int(i64),
    Uint(u64),
}

pub enum FirestorePrimitive {
    Null,
    String(String),
    Number(FirestoreNumber),
}
impl FirestorePrimitive {
    fn into_value(self) -> firestore::Value {
        let kind = match self {
            Self::Null => firestore::value::ValueType::NullValue(0),
            Self::String(string) => firestore::value::ValueType::StringValue(string),
            Self::Number(num) => return num.to_value(),
        };

        firestore::Value {
            value_type: Some(kind),
        }
    }
}

impl<S> From<S> for FirestorePrimitive
where
    S: Into<FirestoreNumber>,
{
    fn from(s: S) -> Self {
        Self::Number(s.into())
    }
}

impl<S> From<Option<S>> for FirestorePrimitive
where
    S: Into<FirestorePrimitive>,
{
    fn from(opt: Option<S>) -> Self {
        match opt {
            Some(val) => val.into(),
            None => Self::Null,
        }
    }
}

impl From<&str> for FirestorePrimitive {
    fn from(string: &str) -> Self {
        Self::String(string.to_owned())
    }
}

impl From<&String> for FirestorePrimitive {
    fn from(string: &String) -> Self {
        Self::String(string.clone())
    }
}

impl From<String> for FirestorePrimitive {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<Cow<'_, str>> for FirestorePrimitive {
    fn from(cow_str: Cow<'_, str>) -> Self {
        Self::String(cow_str.into_owned())
    }
}

impl FirestoreNumber {
    pub fn to_value(self) -> firestore::Value {
        let kind = match self {
            Self::Float(float) => firestore::value::ValueType::DoubleValue(float),
            Self::Int(int) => firestore::value::ValueType::IntegerValue(int),
            Self::Uint(uint) => {
                let int: i64 = match uint.try_into() {
                    Ok(int) => int,
                    Err(_) => {
                        warn!("encountered overflow converting from a u64 -> i64, using i64::MAX");
                        i64::MAX
                    }
                };

                firestore::value::ValueType::IntegerValue(int)
            }
        };

        firestore::Value {
            value_type: Some(kind),
        }
    }
}

macro_rules! impl_firestore_number_from {
    ($($variant:ident => $num_ty:ty as $as_ty:ty),* $(,)?) => {
        $(
            impl From<$num_ty> for FirestoreNumber {
                fn from(num: $num_ty) -> Self {
                    Self::$variant(num as $as_ty)
                }
            }
        )*
    };
}

impl_firestore_number_from! {
    Float => f64 as f64,
    Float => f32 as f64,
    Int => isize as i64,
    Int => i64 as i64,
    Int => i32 as i64,
    Int => i16 as i64,
    Int => i8 as i64,
    Uint => usize as u64,
    Uint => u64 as u64,
    Uint => u32 as u64,
    Uint => u16 as u64,
    Uint => u8 as u64,
}

impl FieldTransformBuilder {
    pub fn field<S>(field: S) -> Self
    where
        S: AsRef<str>,
    {
        let field = ser::escape_field_path(field.as_ref());

        Self { field }
    }

    pub fn set_to_server_time(self) -> FieldTransform {
        FieldTransform {
            field_path: self.field,
            transform_type: Some(TransformType::SetToServerValue(
                ServerValue::RequestTime as i32,
            )),
        }
    }

    pub fn maximum<V>(self, value: V) -> FieldTransform
    where
        FirestoreNumber: From<V>,
    {
        let value = FirestoreNumber::from(value).to_value();

        FieldTransform {
            field_path: self.field,
            transform_type: Some(TransformType::Maximum(value)),
        }
    }

    pub fn minimum<V>(self, value: V) -> FieldTransform
    where
        FirestoreNumber: From<V>,
    {
        let value = FirestoreNumber::from(value).to_value();

        FieldTransform {
            field_path: self.field,
            transform_type: Some(TransformType::Minimum(value)),
        }
    }

    pub fn increment<V>(self, value: V) -> FieldTransform
    where
        FirestoreNumber: From<V>,
    {
        let value = FirestoreNumber::from(value).to_value();

        FieldTransform {
            field_path: self.field,
            transform_type: Some(TransformType::Increment(value)),
        }
    }

    pub fn array_union<I, S>(self, iter: I) -> FieldTransform
    where
        I: IntoIterator<Item = S>,
        FirestorePrimitive: From<S>,
    {
        let values = iter
            .into_iter()
            .map(|e| FirestorePrimitive::from(e).into_value())
            .collect::<Vec<firestore::Value>>();

        let array = firestore::ArrayValue { values };

        FieldTransform {
            field_path: self.field,
            transform_type: Some(TransformType::AppendMissingElements(array)),
        }
    }

    pub fn array_difference<I, S>(self, iter: I) -> FieldTransform
    where
        I: IntoIterator<Item = S>,
        FirestorePrimitive: From<S>,
    {
        let values = iter
            .into_iter()
            .map(|e| FirestorePrimitive::from(e).into_value())
            .collect::<Vec<firestore::Value>>();

        let array = firestore::ArrayValue { values };

        FieldTransform {
            field_path: self.field,
            transform_type: Some(TransformType::RemoveAllFromArray(array)),
        }
    }
}
