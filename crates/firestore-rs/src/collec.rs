//! Contains types to work with Firestore collections.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use protos::firestore::list_documents_request::ConsistencySelector;
use protos::firestore::{
    CreateDocumentRequest, Document, DocumentMask, ListDocumentsRequest, ListDocumentsResponse,
};

use crate::batch::read::{BatchRead, BatchReadStream, Collection, RawBatchReadStream};
use crate::doc::{Doc, DocumentRef};
use crate::query::QueryBuilder;
use crate::ser::serialize_doc_fields;
use crate::{Firestore, PathComponent, Reference};

/// Reference to a firestore collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionRef<C: PathComponent> {
    collection_name: C,
    qualified_path: String,
    pub(super) parent: Firestore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionParent {
    Document(DocumentRef<String, String>),
    Root(Firestore),
}

impl<C: PathComponent> CollectionRef<C> {
    #[inline]
    pub(crate) fn new_root(collection_name: C, parent: Firestore) -> Self {
        Self::new_nested(
            collection_name,
            Reference::new_string(format!("{}/documents", parent.qualified_db_path())),
            parent,
        )
    }

    #[inline]
    pub(crate) fn new_nested(
        collection_name: C,
        parent_path: Box<Reference>,
        parent: Firestore,
    ) -> Self {
        let mut parent_path = parent_path.into_string();
        collection_name.append_to_path(&mut parent_path);
        Self {
            collection_name,
            qualified_path: parent_path,
            parent,
        }
    }

    pub fn doc<D: PathComponent>(&self, doc_id: D) -> DocumentRef<C, D>
    where
        C: Clone,
    {
        DocumentRef::new(CollectionRef::clone(self), doc_id)
    }

    pub(crate) async fn get_docs_inner(
        &mut self,
        mask: Option<DocumentMask>,
        consistency_selector: Option<ConsistencySelector>,
    ) -> crate::Result<ListDocumentsResponse> {
        let list_doc_request = ListDocumentsRequest {
            parent: self.parent_path().to_owned(),
            collection_id: self.collection_name.to_string(),
            page_size: 1000,
            page_token: String::new(),
            order_by: String::new(),
            mask,
            show_missing: false,
            consistency_selector,
        };

        let response = self
            .parent
            .client
            .get()
            .list_documents(list_doc_request)
            .await?;

        Ok(response.into_inner())
    }

    pub async fn get(mut self) -> crate::Result<Vec<Document>> {
        self.get_docs_inner(None, None)
            .await
            .map(|resp| resp.documents)
    }

    async fn push_doc_inner<N, D>(&mut self, id: Option<String>, doc: &D) -> crate::Result<Doc<D>>
    where
        D: serde::de::DeserializeOwned + serde::Serialize,
        N: crate::ser::NullStrategy,
    {
        let doc_fields = serialize_doc_fields::<D, N>(doc)?;

        let request = CreateDocumentRequest {
            parent: self.parent_path().to_owned(),
            collection_id: self.collection_name.to_string(),
            // even though this field is technically optional, prost generated it as a
            // non-optional string?
            document_id: id.unwrap_or_default(),
            mask: None,
            document: Some(Document {
                name: String::new(),
                fields: doc_fields.fields,
                create_time: None,
                update_time: None,
            }),
        };

        let doc_resp = self
            .parent
            .client
            .get()
            .create_document(request)
            .await?
            .into_inner();

        Doc::from_document(doc_resp)
    }

    pub async fn push_doc_with_id<S, D>(&mut self, id: S, doc: &D) -> crate::Result<Doc<D>>
    where
        S: AsRef<str>,
        D: serde::Serialize + serde::de::DeserializeOwned,
    {
        self.push_doc_inner::<crate::ser::NullOverwrite, D>(Some(id.as_ref().to_owned()), doc)
            .await
    }

    pub async fn push_doc<D>(&mut self, doc: &D) -> crate::Result<Doc<D>>
    where
        D: serde::Serialize + serde::de::DeserializeOwned,
    {
        self.push_doc_inner::<crate::ser::NullOverwrite, D>(None, doc)
            .await
    }

    pub async fn push_doc_omit_nulls<D>(&mut self, doc: &D) -> crate::Result<Doc<D>>
    where
        D: serde::Serialize + serde::de::DeserializeOwned,
    {
        self.push_doc_inner::<crate::ser::OmitNulls, D>(None, doc)
            .await
    }

    pub fn query(&mut self) -> QueryBuilder<'_> {
        let parent_path = self.parent_path().to_owned();
        let collection_name = self.collection_name.to_string();
        QueryBuilder::collection_scoped(&mut self.parent.client, parent_path, collection_name)
    }

    pub fn parent_path(&self) -> &str {
        self.qualified_path
            .trim_end_matches('/')
            .rsplit_once('/')
            .expect("having a path with no separators is an invalid state")
            .0
    }

    pub fn parent(&self) -> CollectionParent {
        let parent_path = self.parent_path();

        if parent_path != self.parent.qualified_db_path()
            && let Some((doc_path, doc_id)) = parent_path.rsplit_once('/')
            && let Some((collec_path, collec_name)) = doc_path.rsplit_once('/')
        {
            let grandparent = CollectionRef::new_nested(
                collec_name.to_owned(),
                Reference::new(collec_path).to_owned(),
                self.parent.clone(),
            );
            CollectionParent::Document(grandparent.doc(doc_id.to_owned()))
        } else {
            CollectionParent::Root(self.parent.clone())
        }
    }

    #[inline]
    pub fn name(&self) -> &C {
        &self.collection_name
    }

    #[inline]
    pub fn qualified_path(&self) -> &str {
        self.qualified_path.as_str()
    }

    // listeners dont work yet, they need another pass of dev time
    /*
    #[allow(dead_code)]
    async fn listen(&mut self) -> crate::Result<Listener>
    where
        C: PathComponent
    {
        let database = crate::common::database_path_from_resource_path(&self.qualified_path)?;

        let selector = CollectionSelector {
            collection_id: self.collection_name.to_string(),
            all_descendants: false,
        };

        let query = StructuredQuery {
            from: vec![selector],
            ..Default::default()
        };

        Listener::init_query(&mut self.client, database, self.parent_path.clone(), query).await
    }
    */

    pub fn batch_read(&self) -> BatchRead<Collection> {
        BatchRead::new(
            self.parent.client.clone(),
            self.parent_path().to_owned().into(),
            None,
        )
    }

    pub async fn get_many_raw<I>(&self, doc_ids: I) -> crate::Result<RawBatchReadStream>
    where
        I: IntoIterator,
        I::Item: PathComponent,
    {
        self.batch_read().docs(doc_ids).get_raw().await
    }

    pub async fn get_many<I, O>(&self, doc_ids: I) -> crate::Result<BatchReadStream<O>>
    where
        I: IntoIterator,
        I::Item: PathComponent,
        O: serde::de::DeserializeOwned,
    {
        self.batch_read().docs(doc_ids).get().await
    }

    pub fn get_all_raw(&self) -> RawGetAllStream
    where
        C: PathComponent + Send + 'static,
        Self: Clone,
    {
        #[inline]
        fn to_ok(d: Vec<Document>) -> crate::Result<Vec<Document>> {
            Ok(d)
        }

        RawGetAllStream::new(Self::clone(self), None, to_ok)
    }

    pub fn get_all<D>(&self) -> GetAllStream<Vec<D>>
    where
        D: serde::de::DeserializeOwned + Send + 'static,
        C: PathComponent + Send + 'static,
        Self: Clone,
    {
        GetAllStream::new(Self::clone(self), None, deserialize_docs)
    }

    pub async fn list_documents(&mut self) -> crate::Result<Vec<String>> {
        let req = ListDocumentsRequest {
            parent: format!("{}/documents", self.parent.qualified_db_path()),
            collection_id: self.collection_name.to_string(),
            order_by: String::from("name"),
            mask: Some(DocumentMask {
                field_paths: vec![],
            }),
            show_missing: false,
            page_size: 1000,
            page_token: String::new(),
            consistency_selector: None,
        };

        let resp = self
            .parent
            .client
            .get()
            .list_documents(req)
            .await?
            .into_inner();
        let mut next_page_token = resp.next_page_token;

        let mut dst = if next_page_token.is_empty() {
            Vec::with_capacity(resp.documents.len() * 2)
        } else {
            Vec::with_capacity(resp.documents.len())
        };

        fn push_docs(dst: &mut Vec<String>, docs: Vec<Document>) {
            dst.reserve(docs.len());
            dst.extend(docs.into_iter().map(|doc| doc.name));
        }

        push_docs(&mut dst, resp.documents);

        while !next_page_token.is_empty() {
            let req = ListDocumentsRequest {
                parent: format!("{}/documents", self.parent.qualified_db_path()),
                collection_id: self.collection_name.to_string(),
                order_by: String::from("name"),
                mask: Some(DocumentMask {
                    field_paths: vec!["__name__".to_owned()],
                }),
                show_missing: false,
                page_size: 1000,
                page_token: std::mem::take(&mut next_page_token),
                consistency_selector: None,
            };

            let resp = self
                .parent
                .client
                .get()
                .list_documents(req)
                .await?
                .into_inner();
            next_page_token = resp.next_page_token;

            push_docs(&mut dst, resp.documents);
        }

        dst.shrink_to_fit();
        Ok(dst)
    }

    pub async fn delete(&mut self) -> crate::Result<usize> {
        let to_delete = self.list_documents().await?;

        let writes = to_delete
            .into_iter()
            .map(|doc_path| protos::firestore::Write {
                operation: Some(protos::firestore::write::Operation::Delete(doc_path)),
                update_mask: None,
                current_document: None,
                update_transforms: vec![],
            })
            .collect();

        let req = protos::firestore::BatchWriteRequest {
            writes,
            labels: Default::default(),
            database: self.parent.qualified_db_path().to_owned(),
        };

        let resp = self
            .parent
            .client
            .get()
            .batch_write(req)
            .await?
            .into_inner();
        Ok(resp.write_results.len())
    }
}

#[derive(Debug)]
#[pin_project::pin_project]
pub struct GetAllStream<D> {
    rx: tokio::sync::mpsc::UnboundedReceiver<D>,
    joined: bool,
    #[pin]
    handle: Option<tokio::task::JoinHandle<crate::Result<()>>>,
}

type RawGetAllStream = GetAllStream<Vec<Document>>;

impl<D> GetAllStream<D>
where
    D: Send + 'static,
{
    fn new<C>(
        collec: CollectionRef<C>,
        order_by: Option<String>,
        map_fn: fn(Vec<Document>) -> crate::Result<D>,
    ) -> Self
    where
        C: PathComponent + Send + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            let mut page_token = String::new();

            loop {
                let list_doc_request = ListDocumentsRequest {
                    parent: collec.parent_path().to_owned(),
                    collection_id: collec.collection_name.to_string(),
                    page_size: 1000,
                    page_token: std::mem::take(&mut page_token),
                    order_by: order_by.clone().unwrap_or_default(),
                    mask: None,
                    show_missing: false,
                    consistency_selector: None,
                };

                let response = collec
                    .parent
                    .client
                    .get()
                    .list_documents(list_doc_request)
                    .await?
                    .into_inner();

                if !response.documents.is_empty() {
                    let mapped = map_fn(response.documents)?;
                    if tx.send(mapped).is_err() {
                        // if the reciever has been dropped, there's no way to even
                        // await this join handle, so we can just bail.
                        break;
                    }
                }

                if response.next_page_token.is_empty() {
                    break;
                }

                page_token = response.next_page_token;
            }

            Ok(()) as crate::Result<()>
        });

        Self {
            rx,
            joined: false,
            handle: Some(handle),
        }
    }
}

impl<D> futures::Stream for GetAllStream<D>
where
    D: Send,
{
    type Item = crate::Result<D>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if let Some(result) = ready!(this.rx.poll_recv(cx)) {
            return Poll::Ready(Some(Ok(result)));
        }

        if !*this.joined
            && let Some(handle) = this.handle.as_pin_mut()
        {
            let result = ready!(handle.poll(cx));
            // once it completes, set this to true so we don't poll it again (causes a panic)
            *this.joined = true;

            match result {
                Ok(Err(error)) => return Poll::Ready(Some(Err(error))),
                Err(_) => {
                    return Poll::Ready(Some(Err(crate::Error::Internal("internal task error"))));
                }
                _ => (),
            }
        }

        Poll::Ready(None)
    }
}

fn deserialize_doc<D>(doc: Document) -> crate::Result<D>
where
    D: serde::de::DeserializeOwned,
{
    crate::de::deserialize_doc_fields(doc.fields).map_err(crate::Error::from)
}

fn deserialize_docs<D>(docs: Vec<Document>) -> crate::Result<Vec<D>>
where
    D: serde::de::DeserializeOwned,
{
    let mut dst = Vec::with_capacity(docs.len());

    for result in docs.into_iter().map(deserialize_doc) {
        let deserialized = result?;
        dst.push(deserialized);
    }

    Ok(dst)
}
