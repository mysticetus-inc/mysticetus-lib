use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, ready};

use futures::Stream;
use protos::firestore::batch_get_documents_request::ConsistencySelector;
use protos::firestore::{
    BatchGetDocumentsRequest, BatchGetDocumentsResponse, batch_get_documents_response,
};
use serde::de::DeserializeOwned;
use tonic::Streaming;

use crate::Reference;
use crate::client::FirestoreClient;
use crate::de::deserialize_doc_fields;
use crate::error::Error;

const DEFAULT_READ_CAPACITY: usize = 10;

pub struct BatchRead<B> {
    client: FirestoreClient,
    base_path: B,
    doc_ids: Vec<String>,
}

pub trait BatchReadBase {
    fn get_base(&self) -> &str;

    fn get_database_path(&self) -> Result<String, Error>;
}

pub enum Document {
    QualifiedDbPath(Arc<str>),
    DocumentRoot(String),
}

impl BatchReadBase for Document {
    fn get_base(&self) -> &str {
        match self {
            Self::QualifiedDbPath(db_path) => db_path,
            Self::DocumentRoot(collec_path) => collec_path.as_str(),
        }
    }

    fn get_database_path(&self) -> Result<String, Error> {
        match self {
            Self::QualifiedDbPath(db_path) => Ok(db_path.to_string()),
            Self::DocumentRoot(collec_path) => {
                crate::try_extract_database_path(collec_path.as_str())
                    .map(ToOwned::to_owned)
                    .ok_or(Error::Internal("Unable to get root database path"))
            }
        }
    }
}

impl From<Arc<str>> for Document {
    fn from(s: Arc<str>) -> Self {
        Self::QualifiedDbPath(s)
    }
}

impl From<String> for Document {
    fn from(s: String) -> Self {
        Self::DocumentRoot(s)
    }
}

pub struct Collection(String);

impl BatchReadBase for Collection {
    fn get_base(&self) -> &str {
        self.0.as_str()
    }

    fn get_database_path(&self) -> Result<String, Error> {
        crate::try_extract_database_path(self.0.as_str())
            .map(ToOwned::to_owned)
            .ok_or(Error::Internal("Unable to get root database path"))
    }
}

impl From<String> for Collection {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl<B> BatchRead<B>
where
    B: BatchReadBase,
{
    pub(crate) fn new<C>(client: FirestoreClient, base_path: B, id_capacity: C) -> Self
    where
        C: Into<Option<usize>>,
    {
        let capacity = id_capacity.into().unwrap_or(DEFAULT_READ_CAPACITY);

        Self {
            client,
            base_path,
            doc_ids: Vec::with_capacity(capacity),
        }
    }

    async fn get_inner(
        &mut self,
        consistency_selector: Option<ConsistencySelector>,
    ) -> crate::Result<Streaming<BatchGetDocumentsResponse>> {
        let database = self.base_path.get_database_path()?;

        let request = BatchGetDocumentsRequest {
            database,
            documents: std::mem::take(&mut self.doc_ids),
            mask: None,
            consistency_selector,
        };

        let stream = self
            .client
            .get()
            .batch_get_documents(request)
            .await?
            .into_inner();

        Ok(stream)
    }

    pub async fn get_raw(&mut self) -> Result<RawBatchReadStream, Error> {
        let max = self.doc_ids.len();
        let stream = self.get_inner(None).await?;
        Ok(RawBatchReadStream { stream, max })
    }

    pub async fn get<O>(&mut self) -> Result<BatchReadStream<O>, Error>
    where
        O: DeserializeOwned,
    {
        Ok(self.get_raw().await?.deserialize_stream())
    }
}

impl BatchRead<Document> {
    pub fn collection<S>(&mut self, collection: S) -> BatchReadCollectionRef<'_, S>
    where
        S: fmt::Display,
    {
        BatchReadCollectionRef {
            ctx: self,
            collection,
        }
    }
}

impl BatchRead<Collection> {
    pub fn doc<D>(&mut self, doc_id: D) -> &mut Self
    where
        D: fmt::Display,
    {
        let mut full_doc_path = self.base_path.get_base().to_owned();

        fmt::write(&mut full_doc_path, format_args!("/{}", doc_id))
            .expect("'<String as std::fmt::Display>::fmt' should never fail");

        self.doc_ids.push(full_doc_path);
        self
    }

    pub fn docs<I, D>(&mut self, doc_ids: I) -> &mut Self
    where
        I: IntoIterator<Item = D>,
        D: fmt::Display,
    {
        let iter = doc_ids.into_iter();

        let (low, high) = iter.size_hint();
        let reserve = high.unwrap_or(low);

        let empty_cap = self.doc_ids.capacity() - self.doc_ids.len();
        if empty_cap < reserve {
            self.doc_ids.reserve(reserve - empty_cap);
        }

        for id in iter {
            self.doc(id);
        }

        self
    }
}

pub struct BatchReadCollectionRef<'a, C> {
    ctx: &'a mut BatchRead<Document>,
    collection: C,
}

impl<'a, C> BatchReadCollectionRef<'a, C>
where
    C: fmt::Display,
{
    pub fn doc<D>(&mut self, doc_id: D) -> &mut Self
    where
        D: fmt::Display,
    {
        let mut full_doc_path = self.ctx.base_path.get_base().to_owned();

        fmt::write(&mut full_doc_path, format_args!("/{}", self.collection))
            .expect("'<String as std::fmt::Display>::fmt' should never fail");

        fmt::write(&mut full_doc_path, format_args!("/{}", doc_id))
            .expect("'<String as std::fmt::Display>::fmt' should never fail");

        self.ctx.doc_ids.push(full_doc_path);
        self
    }

    pub fn docs<I, D>(&mut self, doc_ids: I) -> &mut Self
    where
        I: IntoIterator<Item = D>,
        D: fmt::Display,
    {
        let iter = doc_ids.into_iter();

        let (low, high) = iter.size_hint();
        let reserve = high.unwrap_or(low);

        let empty_cap = self.ctx.doc_ids.capacity() - self.ctx.doc_ids.len();
        if empty_cap < reserve {
            self.ctx.doc_ids.reserve(reserve - empty_cap);
        }

        for id in iter {
            self.doc(id);
        }

        self
    }
}

impl<D, D2> Extend<D2> for BatchReadCollectionRef<'_, D>
where
    D: fmt::Display,
    D2: fmt::Display,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = D2>,
    {
        self.docs(iter);
    }
}

#[pin_project::pin_project]
pub struct RawBatchReadStream {
    #[pin]
    stream: tonic::Streaming<BatchGetDocumentsResponse>,
    max: usize,
}

impl RawBatchReadStream {
    pub fn deserialize_stream<O>(self) -> BatchReadStream<O>
    where
        O: DeserializeOwned,
    {
        BatchReadStream {
            stream: self.stream,
            max: self.max,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Stream for RawBatchReadStream {
    type Item = Result<RawReadResult, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        let response = match ready!(this.stream.poll_next(cx)) {
            Some(Ok(resp)) => resp,
            Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
            None => return Poll::Ready(None),
        };

        match response.result {
            Some(res) => {
                *this.max = this.max.saturating_sub(1);
                Poll::Ready(Some(Ok(res.into())))
            }
            // should never be none (I think its a result of weird prost type generation),
            // so skip this item if we run into this.
            None => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.max))
    }
}

pub enum RawReadResult {
    Found(crate::RawDoc),
    Missing(String),
}

impl From<batch_get_documents_response::Result> for RawReadResult {
    fn from(raw: batch_get_documents_response::Result) -> Self {
        match raw {
            batch_get_documents_response::Result::Found(doc) => Self::Found(doc.into()),
            batch_get_documents_response::Result::Missing(doc) => Self::Missing(doc),
        }
    }
}

impl RawReadResult {
    fn path(&self) -> &Reference {
        match self {
            Self::Found(doc) => &doc.reference,
            Self::Missing(missing) => Reference::new(missing.as_str()),
        }
    }

    pub fn doc_id(&self) -> &str {
        self.path().id()
    }

    pub fn parent(&self) -> Option<&str> {
        let parent = self.path().as_str().rsplit_once('/')?.0;
        Some(parent.rsplit_once('/')?.1)
    }

    pub fn get_raw(&self) -> Option<&crate::RawDoc> {
        match self {
            Self::Found(doc) => Some(doc),
            _ => None,
        }
    }

    pub fn get_raw_mut(&mut self) -> Option<&mut crate::RawDoc> {
        match self {
            Self::Found(doc) => Some(doc),
            _ => None,
        }
    }

    pub fn into_raw(self) -> Option<crate::RawDoc> {
        match self {
            Self::Found(doc) => Some(doc),
            _ => None,
        }
    }

    pub fn decode<O>(self) -> Result<Option<O>, Error>
    where
        O: serde::de::DeserializeOwned,
    {
        match self.into_raw() {
            Some(raw_doc) => {
                let deserialized = deserialize_doc_fields(raw_doc.into_inner())?;
                Ok(Some(deserialized))
            }
            _ => Ok(None),
        }
    }
}

#[pin_project::pin_project]
pub struct BatchReadStream<O> {
    #[pin]
    stream: tonic::Streaming<BatchGetDocumentsResponse>,
    max: usize,
    _marker: std::marker::PhantomData<O>,
}

impl<O> Stream for BatchReadStream<O>
where
    O: DeserializeOwned + Unpin,
{
    type Item = Result<O, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let response = match ready!(this.stream.poll_next(cx)) {
            Some(Ok(resp)) => resp,
            Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
            None => return Poll::Ready(None),
        };

        let document = match response.result {
            Some(batch_get_documents_response::Result::Found(doc)) => doc,
            // skip missing responses item if we run into this.
            _ => return Poll::Pending,
        };

        *this.max = this.max.saturating_sub(1);

        Poll::Ready(Some(
            deserialize_doc_fields(document.fields).map_err(Error::from),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.max))
    }
}
