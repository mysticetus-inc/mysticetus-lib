use std::borrow::Cow;
use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use timestamp::Timestamp;

use super::Error;
use super::client::Client;
use super::event::EventStream;
use super::path::{OwnedPath, Path};
use super::query::Query;
use super::shallow::Shallow;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct PostResponse {
    name: String,
}

#[derive(Debug, Clone)]
pub struct Ref<'a> {
    path: Path<'a>,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct OwnedRef {
    path: OwnedPath,
    client: Client,
}

// TODO: give ownedref the same ref functions, so there's no need to convert back and forth.
impl OwnedRef {
    pub(crate) fn new(path: OwnedPath, client: Client) -> Self {
        Self { path, client }
    }

    pub fn path(&self) -> &OwnedPath {
        &self.path
    }

    pub fn parent(&self) -> OwnedRef {
        let mut parent = self.path.clone();
        parent.pop();

        Self::new(parent, self.client.clone())
    }

    pub fn into_parent(mut self) -> Self {
        self.path.pop();
        self
    }

    pub fn is_root(&self) -> bool {
        self.path.n_segments() == 0
    }

    pub fn into_root(mut self) -> Self {
        self.path.clear();
        self
    }

    pub fn parent_checked(&self) -> Option<Self> {
        let mut parent = self.path.clone();
        parent.pop()?;

        Some(Self::new(parent, self.client.clone()))
    }

    pub fn into_parent_checked(mut self) -> Result<Self, Self> {
        match self.path.pop() {
            Some(_) => Ok(Self::new(self.path, self.client)),
            None => Err(self),
        }
    }

    pub fn child_disp<D>(&self, disp: D) -> Self
    where
        D: fmt::Display,
    {
        let mut child_path = self.path.clone();
        child_path.push(disp.to_string());

        Self::new(child_path, self.client.clone())
    }

    pub fn into_child_disp<D>(mut self, disp: D) -> Self
    where
        D: fmt::Display,
    {
        self.path.push(disp.to_string());

        Self::new(self.path, self.client)
    }

    /*
    pub async fn listen(&self) -> Result<EventStream, Error> {
        self.client.start_event_stream(&self.path, false).await
    }


    pub async fn listen_shallow(&self) -> Result<EventStream, Error> {
        self.client.start_event_stream(&self.path, true).await
    }
    */

    pub fn child<D>(&self, child: D) -> Self
    where
        D: Into<String>,
    {
        let mut child_path = self.path.clone();

        child_path.push(child.into());

        Self::new(child_path, self.client.clone())
    }

    pub fn into_child<D>(mut self, child: D) -> Self
    where
        D: Into<String>,
    {
        self.path.push(child.into());
        Self::new(self.path, self.client)
    }

    pub(crate) async fn get_inner<O>(&self) -> Result<O, Error>
    where
        O: DeserializeOwned,
    {
        let resp = self.client.get(&self.path, false).await?;
        crate::deserialize(resp).await
    }

    pub(crate) async fn get_shallow_inner<T>(&self) -> Result<Shallow<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        let resp = self.client.get(&self.path, true).await?;
        crate::deserialize(resp).await
    }

    pub async fn get<O>(&self) -> Result<O, Error>
    where
        O: DeserializeOwned,
    {
        self.get_inner().await
    }

    pub async fn get_shallow<T>(&self) -> Result<Shallow<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        self.get_shallow_inner().await
    }

    pub fn query(&self) -> Query<'_, OwnedPath> {
        Query::new(&self.path, &self.client)
    }

    pub fn query_with_param_capacity(&self, capacity: usize) -> Query<'_, OwnedPath> {
        Query::with_capacity(&self.path, &self.client, capacity)
    }

    pub async fn set<B, O>(&self, value: &B) -> Result<O, Error>
    where
        B: Serialize,
        O: DeserializeOwned,
    {
        let resp = self.client.put(&self.path, value).await?;
        crate::deserialize(resp).await
    }

    pub async fn set_server_timestamp(&self) -> Result<Timestamp, Error> {
        let ts_ms: i64 = self.set(&crate::ServerValue::TimeStamp).await?;

        Timestamp::from_millis_checked(ts_ms).map_err(Error::from)
    }

    pub async fn increment<V>(&self, value: V) -> Result<super::FloatOrInt, Error>
    where
        V: Into<crate::ServerValue>,
    {
        self.set(&value.into()).await
    }

    pub async fn update<B, O>(&self, value: &B) -> Result<O, Error>
    where
        B: Serialize,
        O: DeserializeOwned,
    {
        let resp = self.client.patch(&self.path, value).await?;
        crate::deserialize(resp).await
    }

    pub async fn push<B>(&self, value: &B) -> Result<String, Error>
    where
        B: Serialize,
    {
        let resp = self.client.post(&self.path, value).await?;

        let pushed_resp = crate::deserialize::<PostResponse>(resp).await?;

        Ok(pushed_resp.name)
    }

    pub async fn delete(&self) -> Result<(), Error> {
        self.client.delete(&self.path).await?;
        Ok(())
    }

    pub fn back_to_ref(self) -> Ref<'static> {
        Ref {
            path: self.path.into_path(),
            client: self.client,
        }
    }
}

impl<'a> Ref<'a> {
    pub(crate) fn new<P>(path: P, client: Client) -> Self
    where
        P: Into<Path<'a>>,
    {
        Self {
            path: path.into(),
            client,
        }
    }

    pub fn into_owned(self) -> OwnedRef {
        OwnedRef {
            path: self.path.into_owned(),
            client: self.client,
        }
    }

    pub fn path(&self) -> &Path<'_> {
        &self.path
    }

    pub fn parent(&self) -> Ref<'a> {
        let mut parent = self.path.clone();
        parent.pop();

        Ref::new(parent, self.client.clone())
    }

    pub fn into_parent(mut self) -> Self {
        self.path.pop();
        self
    }

    pub fn is_root(&self) -> bool {
        self.path.n_segments() == 0
    }

    pub fn into_root(mut self) -> Self {
        self.path.clear();
        self
    }

    pub fn parent_checked(&self) -> Option<Ref<'a>> {
        let mut parent = self.path.clone();
        parent.pop()?;

        Some(Ref::new(parent, self.client.clone()))
    }

    pub fn into_parent_checked(mut self) -> Result<Ref<'a>, Self> {
        match self.path.pop() {
            Some(_) => Ok(Ref::new(self.path, self.client)),
            None => Err(self),
        }
    }

    pub fn child_disp<D>(&self, disp: D) -> Ref<'a>
    where
        D: fmt::Display,
    {
        let mut child_path = self.path.clone();
        child_path.push_display(disp);

        Ref::new(child_path, self.client.clone())
    }

    pub fn into_child_disp<D>(mut self, disp: D) -> Ref<'a>
    where
        D: fmt::Display,
    {
        self.path.push_display(disp);

        Ref::new(self.path, self.client)
    }

    pub async fn listen(&self) -> Result<EventStream, Error> {
        self.client.start_event_stream(&self.path, false).await
    }

    pub async fn listen_shallow(&self) -> Result<EventStream, Error> {
        self.client.start_event_stream(&self.path, true).await
    }

    pub fn child<D>(&self, child: D) -> Ref<'a>
    where
        D: Into<Cow<'a, str>>,
    {
        let mut child_path = self.path.clone();

        match child.into() {
            Cow::Borrowed(borrowed) => child_path.push_str(borrowed),
            Cow::Owned(owned) => child_path.push_string(owned),
        }

        Ref::new(child_path, self.client.clone())
    }

    pub fn into_child<D>(mut self, child: D) -> Ref<'a>
    where
        D: Into<Cow<'a, str>>,
    {
        match child.into() {
            Cow::Borrowed(borrowed) => self.path.push_str(borrowed),
            Cow::Owned(owned) => self.path.push_string(owned),
        }

        Ref::new(self.path, self.client)
    }

    pub(crate) async fn get_inner<O>(&self) -> Result<O, Error>
    where
        O: DeserializeOwned,
    {
        let resp = self.client.get(&self.path, false).await?;

        crate::deserialize(resp).await
    }

    pub(crate) async fn get_shallow_inner<T>(&self) -> Result<Shallow<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        let resp = self.client.get(&self.path, true).await?;
        crate::deserialize(resp).await
    }

    pub async fn get<O>(&self) -> Result<O, Error>
    where
        O: DeserializeOwned,
    {
        self.get_inner().await
    }

    pub async fn get_shallow<T>(&self) -> Result<Shallow<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        self.get_shallow_inner().await
    }

    pub fn query(&self) -> Query<'_, Path<'_>> {
        Query::new(&self.path, &self.client)
    }

    pub fn query_with_param_capacity(&self, capacity: usize) -> Query<'_, Path<'_>> {
        Query::with_capacity(&self.path, &self.client, capacity)
    }

    pub async fn set<B, O>(&self, value: &B) -> Result<O, Error>
    where
        B: Serialize,
        O: DeserializeOwned,
    {
        let resp = self.client.put(&self.path, value).await?;
        crate::deserialize(resp).await
    }

    pub async fn set_server_timestamp(&self) -> Result<Timestamp, Error> {
        let ts_ms: i64 = self.set(&crate::ServerValue::TimeStamp).await?;

        Timestamp::from_millis_checked(ts_ms).map_err(Error::from)
    }

    pub async fn increment<V>(&self, value: V) -> Result<super::FloatOrInt, Error>
    where
        V: Into<crate::ServerValue>,
    {
        self.set(&value.into()).await
    }

    pub async fn update<B, O>(&self, value: &B) -> Result<O, Error>
    where
        B: Serialize,
        O: DeserializeOwned,
    {
        let resp = self.client.patch(&self.path, value).await?;
        crate::deserialize(resp).await
    }

    pub async fn push<B>(&self, value: &B) -> Result<String, Error>
    where
        B: Serialize,
    {
        let resp = self.client.post(&self.path, value).await?;
        let post_resp = crate::deserialize::<PostResponse>(resp).await?;
        Ok(post_resp.name)
    }

    pub async fn delete(&self) -> Result<(), Error> {
        self.client.delete(&self.path).await?;
        Ok(())
    }
}
