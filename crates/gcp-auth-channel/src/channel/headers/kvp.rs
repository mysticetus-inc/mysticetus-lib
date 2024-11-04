use std::str::FromStr;

use http::header::{HeaderName, HeaderValue};
use tonic::metadata::{AsciiMetadataKey, AsciiMetadataValue};

/// A trait that abstracts a key/value type together, and unifies the static instantiation methods
pub trait KeyValuePair: private::Sealed {
    /// The type of key
    type Key: Clone + FromStr;

    /// The type of value
    type Value: Clone + FromStr;

    /// Assemble a [`Self::Key`] from a static string. May panic if the static string is invalid
    /// (i.e uppercase characters, etc).
    fn key_from_static(key: &'static str) -> Self::Key;

    /// Assemble a [`Self::Value`] from a static string. May panic if the static string is invalid
    fn value_from_static(value: &'static str) -> Self::Value;
}

/// Trait that abstracts inserting headers into a map, for a given request type.
pub trait InsertHeaders<Kvp: KeyValuePair>: private::Sealed {
    fn insert_header(&mut self, key: Kvp::Key, value: Kvp::Value);

    fn reserve(&mut self, additional: usize);

    fn insert_header_by_ref(&mut self, key: &Kvp::Key, value: &Kvp::Value) {
        self.insert_header(key.clone(), value.clone())
    }

    fn insert_headers<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (Kvp::Key, Kvp::Value)>,
    {
        for (key, value) in iter {
            self.insert_header(key, value);
        }
    }

    fn insert_headers_by_ref<'a, I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a (Kvp::Key, Kvp::Value)>,
        Kvp::Key: 'a,
        Kvp::Value: 'a,
    {
        for (key, value) in iter {
            self.insert_header(key.clone(), value.clone())
        }
    }
}

macro_rules! impl_insert_headers {
    ($($kvp:ty => $req:ty; ($ref_fn:ident, $mut_fn:ident)),* $(,)?) => {
        $(
            impl<Body> private::Sealed for $req { }

            impl<Body> InsertHeaders<$kvp> for $req {
                fn insert_header(
                    &mut self,
                    key: <$kvp as KeyValuePair>::Key,
                    value: <$kvp as KeyValuePair>::Value,
                ) {
                    self.$mut_fn().insert(key, value);
                }

                fn reserve(&mut self, additional: usize) {
                    let needed = (self.$ref_fn().capacity() - self.$ref_fn().len())
                        .saturating_sub(additional);

                    if needed > 0 {
                        self.$mut_fn().reserve(needed);
                    }
                }
            }
        )*
    };
}

impl_insert_headers! {
    Http => http::Request<Body>; (headers, headers_mut),
    Grpc => tonic::Request<Body>; (metadata, metadata_mut),
}

mod private {
    pub trait Sealed {}
}

macro_rules! impl_kvp {
    ($(
        $marker:ident($inner_path:literal)=> ($key:ty, $value:ty)
    ),* $(,)?) => {
        $(
            #[doc = "Marker type used to signify the [`"]
            #[doc = $inner_path]
            #[doc = "`] flavor of headers"]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum $marker { }

            impl private::Sealed for $marker { }

            impl KeyValuePair for $marker {
                type Key = $key;
                type Value = $value;

                fn key_from_static(key: &'static str) -> Self::Key {
                    <$key>::from_static(key)
                }

                fn value_from_static(value: &'static str) -> Self::Value {
                    <$value>::from_static(value)
                }
            }
        )*
    };
}

impl_kvp! {
    Http("http::headers") => (HeaderName, HeaderValue),
    Grpc("tonic::metadata") => (AsciiMetadataKey, AsciiMetadataValue),
}
