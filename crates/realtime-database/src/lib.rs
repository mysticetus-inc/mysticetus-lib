#![feature(vec_into_raw_parts, slice_ptr_get, iter_intersperse, fn_traits)]
use std::borrow::Cow;
use std::fmt;

use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};

pub mod builder;
mod client;
pub mod error;
mod event;
pub mod path;
pub mod query;
mod reference;
mod shallow;

use client::Client;
pub use error::Error;
pub use query::Query;
pub use reference::Ref;
pub use shallow::Shallow;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ServerValue {
    TimeStamp,
    IncrementF64(f64),
    IncrementInt(isize),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FloatOrInt {
    Float(f64),
    Int(isize),
}

impl From<f64> for FloatOrInt {
    fn from(float: f64) -> Self {
        Self::Float(float)
    }
}

impl From<isize> for FloatOrInt {
    fn from(int: isize) -> Self {
        Self::Int(int)
    }
}

impl From<f64> for ServerValue {
    fn from(float: f64) -> Self {
        Self::IncrementF64(float)
    }
}

impl From<isize> for ServerValue {
    fn from(int: isize) -> Self {
        Self::IncrementInt(int)
    }
}

impl From<FloatOrInt> for ServerValue {
    fn from(float_or_int: FloatOrInt) -> Self {
        match float_or_int {
            FloatOrInt::Int(int) => Self::IncrementInt(int),
            FloatOrInt::Float(float) => Self::IncrementF64(float),
        }
    }
}

impl Serialize for ServerValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Increment<T> {
            increment: T,
        }

        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_key(".sv")?;

        match *self {
            Self::TimeStamp => map.serialize_value(&"timestamp")?,
            Self::IncrementF64(increment) => map.serialize_value(&Increment { increment })?,
            Self::IncrementInt(increment) => map.serialize_value(&Increment { increment })?,
        }

        map.end()
    }
}

#[derive(Clone)]
pub struct RealtimeDatabase {
    client: Client,
}

impl RealtimeDatabase {
    pub fn builder<'a>() -> builder::RealtimeDbBuilder<'a> {
        builder::RealtimeDbBuilder::new()
    }

    pub async fn from_database_url<'a, D>(database_url: D) -> Result<Self, Error>
    where
        D: Into<Cow<'a, str>>,
    {
        builder::RealtimeDbBuilder::new()
            .database_url(database_url.into())
            .build()
            .await
    }

    pub async fn get<O>(&self) -> Result<O, Error>
    where
        O: DeserializeOwned,
    {
        Ref::new("", self.client.clone()).get().await
    }

    pub async fn get_shallow<T>(&self) -> Result<Shallow<T>, Error>
    where
        T: DeserializeOwned + Ord,
    {
        Ref::new("", self.client.clone()).get_shallow().await
    }

    /// Returns the database url this client is configured with.
    pub fn database_url(&self) -> &str {
        &self.client.db_url
    }

    /// Returns a reference to a child, at a given path. If the given path includes
    /// a '/', it's the same as calling 'child' on each component.
    pub fn child_disp<C>(&self, path: C) -> Ref<'_>
    where
        C: fmt::Display,
    {
        Ref::new(path.to_string(), self.client.clone())
    }

    pub fn child<'a, C>(&self, path: C) -> Ref<'a>
    where
        C: Into<Cow<'a, str>>,
    {
        Ref::new(path.into(), self.client.clone())
    }
}

impl fmt::Debug for RealtimeDatabase {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // defer to the client debug function, with the proper struct name
        self.client.debug_fmt("RealtimeDatabase", formatter)
    }
}

pub(crate) async fn deserialize<O>(response: reqwest::Response) -> Result<O, Error>
where
    O: DeserializeOwned,
{
    let bytes = response.bytes().await?;

    let mut de = serde_json::Deserializer::from_slice(&bytes);

    let wrapped_de = path_aware_serde::Deserializer::new(&mut de);

    let output = O::deserialize(wrapped_de).map_err(Error::de)?;

    de.end().map_err(Error::de)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gcp_auth_provider::Auth;

    use super::*;

    const GUID: &str = "0ed284b2-c9a8-41fe-a846-2fce914a2d50";

    lazy_static::lazy_static! {
        static ref AUTH: Auth = {
            todo!()
            //Auth::new_from_service_account_file(PROJECT_ID, CERT, gcp_auth_channel::Scope::FirestoreRealtimeDatabase).unwrap()
        };
    }

    #[tokio::test]
    async fn test_get_stations() -> Result<(), Error> {
        let db = RealtimeDatabase::builder()
            .with_auth_manager(AUTH.clone())
            .build()
            .await?;

        let stations = db.child("stations").get::<serde_json::Value>().await?;

        let mut f = std::fs::File::create("geojson-stations.json").unwrap();
        serde_json::to_writer_pretty(&mut f, &stations).unwrap();
        Ok(())
    }

    #[derive(Default, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
    struct TestStruct {
        array: Vec<u32>,
        string: String,
        null: Option<()>,
        map: HashMap<String, TestStruct>,
    }

    impl TestStruct {
        fn rand() -> Self {
            let mut map = HashMap::with_capacity(1);
            map.insert("default".to_owned(), Self::default());

            Self {
                array: vec![1, 2, 3],
                string: String::from("test"),
                null: None,
                map,
            }
        }
    }

    #[tokio::test]
    async fn test_event_stream() -> Result<(), Error> {
        use futures::StreamExt;

        let db = RealtimeDatabase::builder()
            .with_auth_manager(AUTH.clone())
            .build()
            .await?;

        let mut event_stream = db.child("test").listen().await?;

        while let Some(result) = event_stream.next().await {
            let event = result?;

            println!("{event:#?}");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test() -> Result<(), Error> {
        let db = RealtimeDatabase::builder()
            .with_auth_manager(AUTH.clone())
            .build()
            .await?;

        let path_data: Shallow<String> = db.child("status").get_shallow().await?;
        println!("{:#?}", path_data);

        let test_set = TestStruct::rand();
        db.child("test").child("test2").push(&test_set).await?;

        let server_value = db.child("server-value");

        let ts = server_value
            .child("timestamp")
            .set_server_timestamp()
            .await?;
        let incr_float = server_value.child("increment-float").increment(0.1).await?;
        let incr_int = server_value.child("increment-int").increment(2).await?;

        println!("server timestamp: {:#?}", ts);
        println!("server incr float: {:#?}", incr_float);
        println!("server incr int: {:#?}", incr_int);

        Ok(())
    }

    #[tokio::test]
    async fn test_key_helpers() -> Result<(), Error> {
        let db = RealtimeDatabase::builder()
            .with_auth_manager(AUTH.clone())
            .build()
            .await?;

        let result = db
            .child("tracklines")
            .child(GUID)
            .query()
            .first_last_key::<timestamp::Timestamp>()
            .await?;

        println!("{result:#?}");

        Ok(())
    }

    #[tokio::test]
    async fn test_query() -> Result<(), Error> {
        let db = RealtimeDatabase::builder()
            .with_auth_manager(AUTH.clone())
            .build()
            .await?;

        let result: Shallow<String> = db
            .child("tracklines")
            .child(GUID)
            .query()
            .limit_to_first(1)
            .get_shallow()
            .await?;

        println!("{result:#?}");

        Ok(())
    }
}
