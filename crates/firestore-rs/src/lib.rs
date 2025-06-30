#![feature(let_chains)]

//! Wrapper around the gRPC Firestore API that emulates the Javascript Firestore API.
//!
//! ## Examples:
//!
//! ```no_run
//! use firestore_rs::{Doc, Firestore};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
//! struct User {
//!     id: String,
//!     name: String,
//!     // ...
//! }
//!
//! let mut firestore = Firestore::new("project_id").await?;
//!
//! // Setting a document:
//! let new_user = User {
//!     ..Default::default()
//! };
//!
//! firestore
//!     .collection("users")
//!     .doc(&new_user.id)
//!     .set(new_user.clone()) // cloning to use in checks later on
//!     .await?;
//!
//! // Retrieving a document:
//! let retrieved_doc: User = firestore
//!     .collection("users")
//!     .doc(&new_user.id)
//!     .get()
//!     .await?
//!     .expect("we just set the document, so it should exist")
//!     .into_inner();
//!
//! assert_eq!(new_user, retrieved_doc);
//!
//! // Deleting a document:
//! firestore
//!     .collection("users")
//!     .doc(&new_user.id)
//!     .delete()
//!     .await?;
//! ```

#[cfg(feature = "firestore-admin")]
mod admin;

pub mod batch;
mod client;
pub mod collec;
mod common;
pub mod de;
pub mod doc;
pub mod error;
pub mod firestore;
// pub mod write_stream;
// pub mod listen;
mod query;
mod ser;
mod transaction;
mod util;
mod value;

pub mod timestamp;

use std::borrow::Cow;
use std::fmt::{Debug, Display};

pub use error::{ConvertError, Error};

/// Alias to [`std::result::Result`], with the [`Err`] variant already set to [`error::Error`].
pub type Result<O> = std::result::Result<O, Error>;

#[macro_use]
extern crate tracing;

use ::timestamp::Timestamp;
pub use collec::CollectionRef;
pub use doc::{Doc, DocumentRef, RawDoc};
pub use firestore::Firestore;
pub use protos::r#type::LatLng;
pub use ser::{DocFields, escape_field_path_into};
pub use value::{Reference, Value};

/// Wrapper/marker types for field transforms.
pub mod transform {
    pub use crate::ser::field_transform::{
        ArrayUnion, Increment, Maximum, Minimum, RemoveFromArray, ServerTimestamp,
    };
}

/// Shared trait to represent collection names and document ids.
///
/// Used to allow for [`str`]/[`String`] based types + others in one impl.
///
/// A [`ToOwned`] bound is used over a [`Clone`] bound to make a [`str`] (with no reference)
/// impl work, along with the blanket:
/// `impl<T: PathComponent> PathComponent for &T { /* ... */ }`
pub trait PathComponent: PartialEq + ToOwned + Display + Debug {
    fn append_to_path(&self, path: &mut String);
}

impl<T> PathComponent for &T
where
    T: PathComponent + ?Sized,
{
    #[inline]
    fn append_to_path(&self, path: &mut String) {
        T::append_to_path(self, path)
    }
}

impl<T> PathComponent for Box<T>
where
    T: PathComponent + Clone + ?Sized,
{
    #[inline]
    fn append_to_path(&self, path: &mut String) {
        T::append_to_path(self, path)
    }
}

impl<T> PathComponent for std::sync::Arc<T>
where
    T: PathComponent + ?Sized,
{
    #[inline]
    fn append_to_path(&self, path: &mut String) {
        T::append_to_path(self, path)
    }
}

macro_rules! impl_str_path_component {
    ($($type:ty),* $(,)?) => {
        $(
            impl PathComponent for $type {
                #[inline]
                fn append_to_path(&self, path: &mut String) {
                    let self_has_sep = self.starts_with('/');
                    let path_has_sep = path.ends_with('/');

                    if self_has_sep && path_has_sep {
                        path.pop();
                    } else if !self_has_sep && !path_has_sep {
                        path.push_str("/");
                    }

                    path.push_str(self.as_ref());
                }
            }
        )*
    };
}

impl_str_path_component! {
    str,
    String,
    Cow<'_, str>,
}

impl PathComponent for uuid::Uuid {
    fn append_to_path(&self, path: &mut String) {
        if !path.ends_with('/') {
            path.push('/');
        }
        // reserve the 36 bytes needed for a string formatted uuid (with hyphens)
        path.reserve(36);
        std::fmt::write(path, format_args!("{self}"))
            .expect("<String as std::fmt::Write>::write should never fail")
    }
}

pub enum NullBehavior {
    NeverWrite,
    WriteOnSet,
    WriteOnUpdate,
}

/*
// TODO: Fix conflicting impls
impl<T> From<T> for Value
where
    T: ToValue
{
    #[inline]
    fn from(x: T) -> Self {
        x.to_value()
    }
}
*/

pub trait ToValue {
    fn to_value(self) -> Value;
}

macro_rules! impl_noop_to_value {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl ToValue for $t {
                fn to_value(self) -> Value {
                    Value::$variant(self)
                }
            }
        )*
    };
}

impl_noop_to_value! {
    i64 => Integer,
    f64 => Double,
    bool => Bool,
    Timestamp => Timestamp,
    Box<Reference> => Reference,
    String => String,
    bytes::Bytes => Bytes,
    LatLng => GeoPoint,
}

impl ToValue for Vec<u8> {
    fn to_value(self) -> Value {
        Value::Bytes(self.into())
    }
}

impl ToValue for value::Value {
    #[inline]
    fn to_value(self) -> value::Value {
        self
    }
}

impl<T: ToValue> ToValue for Option<T> {
    #[inline]
    fn to_value(self) -> Value {
        match self {
            Some(value) => value.to_value(),
            None => Value::Null,
        }
    }
}

impl ToValue for protos::firestore::Value {
    #[inline]
    fn to_value(self) -> Value {
        Value::from_proto_value(self)
    }
}

impl ToValue for protos::firestore::value::ValueType {
    #[inline]
    fn to_value(self) -> Value {
        Value::from_proto_value_type(self)
    }
}

impl ToValue for f32 {
    #[inline]
    fn to_value(self) -> Value {
        Value::Double(self as f64)
    }
}

macro_rules! impl_int_to_value {
    ($($int:ty),* $(,)?) => {
        $(
            impl ToValue for $int {
                #[inline]
                fn to_value(self) -> Value {
                    Value::Integer(self as i64)
                }
            }
        )*
    };
}

impl_int_to_value!(u16, u32, u64, u128, usize, i8, i16, i32, i128, isize);
/*
impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(self) -> Value {
        let mut dst = Vec::with_capacity(self.len());
        dst.extend(
            self.into_iter()
                .map(ToValue::to_value)
                .map(Value::to_proto_value),
        );

        Value::Array(Array(dst))
    }
}

impl<T: ToValue, const N: usize> ToValue for [T; N] {
    fn to_value(self) -> Value {
        let mut dst = Vec::with_capacity(self.len());
        dst.extend(
            self.into_iter()
                .map(ToValue::to_value)
                .map(Value::to_proto_value),
        );
        Value::Array(Array(dst))
    }
}

impl ToValue for &str {
    fn to_value(self) -> Value {
        Value::String(self.to_owned())
    }
}

impl<T> ToValue for &[T]
where
    for<'a> &'a T: ToValue,
{
    fn to_value(self) -> Value {
        let mut dst = Vec::with_capacity(self.len());
        dst.extend(
            self.into_iter()
                .map(ToValue::to_value)
                .map(Value::to_proto_value),
        );
        Value::Array(Array(dst))
    }
}

impl ToValue for Box<str> {
    fn to_value(self) -> Value {
        Value::String(String::from(self))
    }
}

impl ToValue for Cow<'_, str> {
    fn to_value(self) -> Value {
        Value::String(self.into_owned())
    }
}

impl<T> ToValue for Cow<'_, T>
where
    T: ToOwned,
    for<'a> &'a T: ToValue,
    T::Owned: ToValue,
{
    fn to_value(self) -> Value {
        match self {
            Self::Borrowed(b) => b.to_value(),
            Self::Owned(o) => o.to_value(),
        }
    }
}

impl ToValue for uuid::Uuid {
    fn to_value(self) -> Value {
        Value::String(self.to_string())
    }
}
*/

pub(crate) fn try_extract_database_path(path: &str) -> Option<&str> {
    let mut seps_remaining: usize = 4;

    let find_sep = |ch: char| -> bool {
        if matches!(ch, '/') {
            if let Some(rem) = seps_remaining.checked_sub(1) {
                seps_remaining = rem;
            }

            seps_remaining == 0
        } else {
            false
        }
    };

    let index = path.find(find_sep)?;

    path.get(..index)
}

#[test]
fn test_extract_db_path() {
    let doc_path = "projects/PROJECT_ID/databases/DATABASE_ID/documents/DOC_ID";

    let db_path = try_extract_database_path(doc_path).expect("its bad");

    assert_eq!(db_path, "projects/PROJECT_ID/databases/DATABASE_ID");
}

#[allow(unused_mut, unused_variables)]
#[cfg(test)]
mod tests {
    use tokio::sync::OnceCell;

    use super::*;

    static CLIENT: OnceCell<Firestore> = OnceCell::const_new();

    async fn get_client() -> &'static Firestore {
        async fn init() -> Firestore {
            firestore::Firestore::new("winged-citron-305220", gcp_auth_channel::Scope::Firestore)
                .await
                .expect("should be able to build client")
        }

        CLIENT.get_or_init(init).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dump_data() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures::StreamExt;

        let client = get_client().await;

        let collec = client.collection("frames");

        let stream = collec.get_all_raw();

        futures::pin_mut!(stream);

        let (dump_tx, dump_rx) = std::sync::mpsc::channel::<protos::firestore::Document>();

        let rx = std::sync::Arc::new(std::sync::Mutex::new(dump_rx));

        let handles = std::iter::repeat_n(rx, 16)
            .map(|rx| {
                std::thread::spawn(move || {
                    let mut docs: usize = 0;
                    while let Ok(doc) = rx.lock().unwrap().recv() {
                        let id = doc
                            .name
                            .rsplit_once("/")
                            .map(|(_, id)| id)
                            .unwrap_or(&doc.name);

                        let values = crate::value::Map::from(doc.fields);

                        let dst = format!("/home/mrudisel/src/horizon/data/{id}.json");
                        let mut f = std::fs::File::create(&dst)?;
                        serde_json::to_writer_pretty(&mut f, &values)?;
                        docs += 1;
                    }

                    Ok(docs) as std::result::Result<usize, Box<dyn std::error::Error + Send + Sync>>
                })
            })
            .collect::<Vec<_>>();

        while let Some(result) = stream.next().await {
            for doc in result? {
                dump_tx.send(doc).unwrap();
            }
        }

        drop(dump_tx);

        let mut total = 0;

        for handle in handles {
            total += handle.join().unwrap()?;
        }

        println!("wrote out {total} docs");

        Ok(())
    }

    #[tokio::test]
    async fn test_transform() -> crate::Result<()> {
        let client = get_client().await;

        let doc = serde_json::json!({
            "a": "value",
            "b": "value2",
        });

        client
            .collection("tests")
            .doc("transforms")
            .build_write()
            .update(&doc)?
            .field_increment("incr", 5)
            .commit()
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_update() -> crate::Result<()> {
        use serde_json::json;

        let client = get_client().await;

        let mut doc_ref = client.collection("tests").doc("test_update");

        let initial = json!({
            "a": "original",
            "b": "deleted",
            "const": "constant",
            "nested": {
                "c": "original",
                "d": ["original"],
                "e": "deleted",
                "const": "constant",
            }
        });

        let update = json!({
            "a": "updated",
            "b": null,
            "nested": {
                "c": "updated",
                "d": ["updated"],
                "e": null,
            }
        });

        let final_expected = json!({
            "a": "updated",
            "const": "constant",
            "b": null,
            "nested": {
                "c": "updated",
                "d": ["updated"],
                "const": "constant",
                "e": null,
            }
        });

        doc_ref.update(&initial).await?;

        let final_resp: serde_json::Value = doc_ref.update(&update).await?.into_inner();

        println!("final: {:#?}", final_resp);
        println!("final_expected: {:#?}", final_expected);

        assert_eq!(final_resp, final_expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_paths() -> crate::Result<()> {
        use serde_json::json;

        let client = get_client().await;

        let doc_json = json!({
            "ok_path": "value",
            "path with space": "value",
            "path'with'ticks": "value",
            "path with'both": "value",
            "nested object": {
                "ok_path": "value",
                "path with space": "value",
                "path'with'ticks": "value",
                "path with'both": "value",
            }
        });

        let returned: serde_json::Value = client
            .collection("tests")
            .doc("test_paths")
            .set(&doc_json)
            .await?
            .into_inner();

        assert_eq!(doc_json, returned);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_path() -> crate::Result<()> {
        use std::collections::HashMap;

        let client = get_client().await;

        let returned: HashMap<String, serde_json::Value> = client
            .collection("tests")
            .doc("get")
            .get()
            .await?
            .unwrap()
            .into_inner();

        println!("{:#?}", returned);

        Ok(())
    }

    #[ignore = "needs to be refactored to use the 'mysticetus-oncloud' project"]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_query() -> crate::Result<()> {
        use std::collections::HashMap;

        use futures::StreamExt;

        let client =
            firestore::Firestore::new("winged-citron-305220", gcp_auth_channel::Scope::Firestore)
                .await?;

        let mut result_stream = client
            .collection("videos")
            .query()
            .where_field("cameraType")
            .equals("infrared")
            .limit(5)
            .run()
            .await?;

        let mut results = Vec::new();

        while let Some(next_result) = result_stream.next().await {
            let doc_opt: Option<HashMap<String, serde_json::Value>> = next_result?;

            if let Some(doc) = doc_opt {
                results.push(doc);
            }
        }

        let target_value = serde_json::Value::String("infrared".into());

        for doc in results.iter() {
            assert_eq!(doc.get("cameraType"), Some(&target_value));
        }

        println!("found {} docs", results.len());

        if let Some(first) = results.first() {
            println!("first doc: {:#?}", first);
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_list_collection_ids() -> crate::Result<()> {
        use futures::StreamExt;

        let mut client = get_client().await.clone();

        let mut id_stream = Box::pin(client.list_collection_ids());
        let mut ids = Vec::new();

        while let Some(result) = id_stream.next().await {
            let new_batch = result?;

            println!("new batch: {:?}", new_batch);

            ids.extend(new_batch);
        }

        assert!(!ids.is_empty());
        assert!(ids.contains(&String::from("tests")));

        Ok(())
    }
}
