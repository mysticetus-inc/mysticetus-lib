use std::borrow::Cow;
use std::fmt;
use std::ops::{Bound, RangeBounds};

use serde::de::DeserializeOwned;
use serde::ser::{self, SerializeSeq, SerializeTuple};

use super::Error;
use super::client::Client;
use super::path::RtDbPath;
use super::shallow::{Shallow, SingleKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderBy<'q> {
    Key,
    Value,
    Priority,
    Field(Cow<'q, str>),
}

impl<'q> OrderBy<'q> {
    pub fn as_pair(&self) -> (&'static str, &str) {
        match self {
            Self::Key => ("orderBy", "\"$key\""),
            Self::Priority => ("orderBy", "\"$priority\""),
            Self::Value => ("orderBy", "\"$value\""),
            Self::Field(field_path) => ("orderBy", &**field_path),
        }
    }
}

impl ser::Serialize for OrderBy<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;

        let (name, value) = self.as_pair();

        tup.serialize_element(name)?;
        tup.serialize_element(value)?;
        tup.end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryParam<'q> {
    StartAt(Cow<'q, str>),
    EndAt(Cow<'q, str>),
    EqualTo(Cow<'q, str>),
    LimitToFirst(usize),
    LimitToLast(usize),
}

impl ser::Serialize for QueryParam<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;

        let (name, value) = self.as_query_pair();

        tup.serialize_element(name)?;

        match value {
            QueryParamValue::Str(string) => tup.serialize_element(&*string)?,
            QueryParamValue::Num(num) => tup.serialize_element(&num)?,
        }

        tup.end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryParamValue<'a> {
    Str(Cow<'a, str>),
    Num(usize),
}

impl<'a> From<&'a str> for QueryParamValue<'a> {
    fn from(s: &'a str) -> Self {
        Self::Str(Cow::Borrowed(s))
    }
}

impl<'a> From<Cow<'a, str>> for QueryParamValue<'a> {
    fn from(cow: Cow<'a, str>) -> Self {
        Self::Str(cow)
    }
}

impl<'a> From<&'a Cow<'a, str>> for QueryParamValue<'a> {
    fn from(cow: &'a Cow<'a, str>) -> Self {
        Self::Str(Cow::Borrowed(&**cow))
    }
}

impl<'a> From<usize> for QueryParamValue<'a> {
    fn from(num: usize) -> Self {
        Self::Num(num)
    }
}

impl<'a> From<&'a String> for QueryParamValue<'a> {
    fn from(string: &'a String) -> Self {
        Self::Str(Cow::Borrowed(string.as_str()))
    }
}

impl<'a> QueryParam<'a> {
    fn as_query_pair(&self) -> (&'static str, QueryParamValue<'_>) {
        match self {
            Self::StartAt(args) => ("startAt", args.into()),
            Self::EndAt(args) => ("endAt", args.into()),
            Self::EqualTo(args) => ("equalTo", args.into()),
            Self::LimitToFirst(num) => ("limitToFirst", (*num).into()),
            Self::LimitToLast(num) => ("limitToLast", (*num).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Query<'q, P> {
    path: &'q P,
    client: &'q Client,
    query: Params<'q>,
}

impl<'q, P> Query<'q, P>
where
    P: RtDbPath + Send + Sync,
{
    pub(crate) fn new(path: &'q P, client: &'q Client) -> Self {
        Query {
            path,
            client,
            query: Params {
                order_by: None,
                params: Vec::new(),
            },
        }
    }

    pub(crate) fn with_capacity(path: &'q P, client: &'q Client, capacity: usize) -> Self {
        Self {
            path,
            client,
            query: Params {
                order_by: None,
                params: Vec::with_capacity(capacity),
            },
        }
    }

    pub async fn get<O>(&self) -> Result<O, Error>
    where
        O: DeserializeOwned,
    {
        self.client
            .get_with_query(self.path, &self.query, false)
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    async fn get_single_key<T>(&self) -> Result<SingleKey<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        let single_key: Option<SingleKey<T>> = self.get().await?;

        Ok(single_key.unwrap_or_default())
    }

    pub async fn get_shallow<T>(&self) -> Result<Shallow<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        let shallow = self.query.order_by.is_some() || !self.query.params.is_empty();

        self.client
            .get_with_query(self.path, &self.query, shallow)
            .await?
            .json()
            .await
            .map_err(Error::from)
    }

    pub async fn first_key<T>(mut self) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        self.query.params.clear();
        self.order_by_key()
            .limit_to_first(1)
            .get_single_key::<T>()
            .await
            .map(|k| k.into_inner())
    }

    pub async fn last_key<T>(mut self) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        self.query.params.clear();
        self.order_by_key()
            .limit_to_last(1)
            .get_single_key::<T>()
            .await
            .map(|k| k.into_inner())
    }

    pub async fn first_last_key<T>(mut self) -> Result<KeyBounds<T>, Error>
    where
        T: DeserializeOwned + Ord + Send,
    {
        // clear before cloning, that way we dont clone any existing contents.
        self.query.params.clear();

        let first = self.clone().first_key().await?;
        let last = self.last_key().await?;

        /*
        // FIXME: this deadlocks, so for now we'll just do the requests sequentially.
        // Since we aren't send, we also can't spawn them in tasks.
        let (first, last) = tokio::try_join!(
            self.clone().first_key(),
            self.last_key(),
        )?;
        */

        match (first, last) {
            (Some(first), Some(last)) => Ok(KeyBounds::Bounded { first, last }),
            (Some(key), _) | (_, Some(key)) => Ok(KeyBounds::SingleKey(key)),
            (None, None) => Ok(KeyBounds::NoKeys),
        }
    }

    pub fn order_by_value(mut self) -> Self {
        self.query.order_by = Some(OrderBy::Value);
        self
    }

    pub fn order_by_key(mut self) -> Self {
        self.query.order_by = Some(OrderBy::Key);
        self
    }

    pub fn order_by_priority(mut self) -> Self {
        self.query.order_by = Some(OrderBy::Priority);
        self
    }

    pub fn order_by_disp<D>(mut self, order_by: &D) -> Self
    where
        D: fmt::Display,
    {
        self.query.order_by = Some(OrderBy::Field(order_by.to_string().into()));
        self
    }

    pub fn order_by<S>(mut self, order_by: S) -> Self
    where
        S: Into<Cow<'q, str>>,
    {
        self.query.order_by = Some(OrderBy::Field(order_by.into()));
        self
    }

    pub fn limit_to_first(mut self, amount: usize) -> Self {
        self.query.params.push(QueryParam::LimitToFirst(amount));
        self
    }

    pub fn limit_to_last(mut self, amount: usize) -> Self {
        self.query.params.push(QueryParam::LimitToLast(amount));
        self
    }

    pub fn key_starts_with<S>(mut self, range: S) -> Self
    where
        S: Into<String>,
    {
        let mut start = range.into();
        start.push('\u{0}');

        let mut end = start.clone();
        end.pop();
        end.push(char::MAX);

        self.query.params.push(QueryParam::StartAt(start.into()));
        self.query.params.push(QueryParam::EndAt(end.into()));
        self
    }

    pub fn range_str<R, S>(mut self, range: R) -> Self
    where
        R: RangeBounds<S>,
        S: AsRef<str> + 'q,
    {
        match range.start_bound() {
            Bound::Included(start) => {
                let start = Cow::Owned(start.as_ref().to_owned());
                self.query.params.push(QueryParam::StartAt(start));
            }
            Bound::Excluded(start) => {
                let mut start_offset = start.as_ref().to_owned();

                let final_char = start_offset.pop();

                let decremented_char = final_char
                    .map(u32::from)
                    .and_then(|ch_u32| ch_u32.checked_sub(1))
                    .and_then(char::from_u32);

                // push the decremented character, or if that's invalid, re-insert the
                // original char
                if let Some(final_char) = decremented_char.or(final_char) {
                    start_offset.push(final_char);
                }

                self.query
                    .params
                    .push(QueryParam::StartAt(Cow::Owned(start_offset)));
            }
            Bound::Unbounded => (),
        }

        match range.end_bound() {
            Bound::Included(end) => {
                let end = Cow::Owned(end.as_ref().to_owned());
                self.query.params.push(QueryParam::EndAt(end));
            }
            Bound::Excluded(end) => {
                let mut end_offset = end.as_ref().to_owned();

                let final_char = end_offset.pop();

                let incremented_char = final_char
                    .map(u32::from)
                    .and_then(|ch_u32| ch_u32.checked_add(1))
                    .and_then(char::from_u32);

                // push the incremented character, or if that's invalid, re-insert the
                // original char
                if let Some(final_char) = incremented_char.or(final_char) {
                    end_offset.push(final_char);
                }

                self.query
                    .params
                    .push(QueryParam::EndAt(Cow::Owned(end_offset)));
            }
            Bound::Unbounded => (),
        }

        self
    }

    /*
    pub fn range_num<R, I>(mut self, range: R) -> Self
    where
        R: RangeBounds<I>,
        I: num::Integer,
    {
        match range.start_bound() {
            Bound::Included(start) => {
                let start = Cow::Owned(start.to_string());
                self.query.params.push(QueryParam::StartAt(start));
            },
            Bound::Excluded(start) => {
                let offset_start = Cow::Owned((start + 1).to_string());
                self.query.params.push(QueryParam::StartAt(offset_start));
            },
            Bound::Unbounded => (),
        }

        match range.end_bound() {
            Bound::Included(end) => {
                let end = Cow::Owned(end.to_string());
                self.query.params.push(QueryParam::EndAt(end));
            },
            Bound::Excluded(end) => {
                let offset_end = Cow::Owned((end - 1).to_string());
                self.query.params.push(QueryParam::EndAt(offset_end));
            },
            Bound::Unbounded => ()
        }

        self
    }
    */
    pub fn end_at_disp<S>(mut self, end_at: &S) -> Self
    where
        S: fmt::Display,
    {
        self.query
            .params
            .push(QueryParam::EndAt(end_at.to_string().into()));
        self
    }

    pub fn end_at<S>(mut self, end_at: S) -> Self
    where
        S: Into<Cow<'q, str>>,
    {
        self.query.params.push(QueryParam::EndAt(end_at.into()));
        self
    }

    pub fn start_at_disp<S>(mut self, start_at: &S) -> Self
    where
        S: fmt::Display,
    {
        self.query
            .params
            .push(QueryParam::StartAt(start_at.to_string().into()));
        self
    }

    pub fn start_at<S>(mut self, start_at: S) -> Self
    where
        S: Into<Cow<'q, str>>,
    {
        self.query.params.push(QueryParam::StartAt(start_at.into()));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyBounds<T> {
    Bounded { first: T, last: T },
    SingleKey(T),
    NoKeys,
}

#[derive(Debug, Clone)]
struct Params<'q> {
    order_by: Option<OrderBy<'q>>,
    params: Vec<QueryParam<'q>>,
}

impl ser::Serialize for Params<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let len = self.params.len() + usize::from(self.order_by.is_some());

        let mut seq = serializer.serialize_seq(Some(len))?;

        if let Some(order_by) = self.order_by.as_ref() {
            seq.serialize_element(order_by)?;
        }

        for param in self.params.iter() {
            seq.serialize_element(param)?;
        }

        seq.end()
    }
}
