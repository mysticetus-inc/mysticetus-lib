//! Firestore document serializers for [`Serialize`]-able types.

use std::borrow::Cow;
use std::collections::HashMap;

use protos::firestore::value::ValueType;
use protos::firestore::{self, DocumentMask, MapValue, Value};
use protos::protobuf::NullValue;
use serde::Serialize;

mod doc;
mod map;
mod value;

pub(crate) use doc::DocSerializer;
use map::MapSerializer;
pub(crate) use value::ValueSerializer;

use crate::ConvertError;

/// A trait for handling null values in either set or update serialization.
pub(crate) trait NullStrategy: Default + Copy + Eq {
    const OMIT: bool;

    /// Given a closure that takes in a value, the function is only called if
    /// we want to insert nulls.
    fn handle_null<F, O>(f: F)
    where
        F: FnOnce(firestore::Value) -> O;
}

/// Ignore all nulls, which prevents overwriting existing values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OmitNulls;

impl NullStrategy for OmitNulls {
    const OMIT: bool = true;

    #[inline]
    fn handle_null<F, O>(_: F)
    where
        F: FnOnce(firestore::Value) -> O,
    {
    }
}

/// Include all nulls, overwriting existing values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NullOverwrite;

impl NullStrategy for NullOverwrite {
    const OMIT: bool = false;

    #[inline]
    fn handle_null<F, O>(f: F)
    where
        F: FnOnce(firestore::Value) -> O,
    {
        f(null_value());
    }
}

pub(crate) fn serialize_doc_fields<T, N>(value: &T) -> crate::Result<DocFields>
where
    T: Serialize + ?Sized,
    N: NullStrategy,
{
    value
        .serialize(DocSerializer::<N>::default())
        .map_err(crate::Error::Convert)
}

pub fn serialize_set_doc<T>(value: &T) -> crate::Result<DocFields>
where
    T: Serialize + ?Sized,
{
    serialize_doc_fields::<T, NullOverwrite>(value)
}

pub fn serialize_update_doc<T>(value: &T) -> crate::Result<DocFields>
where
    T: Serialize + ?Sized,
{
    serialize_doc_fields::<T, OmitNulls>(value)
}

#[inline]
const fn null_value() -> Value {
    Value {
        value_type: Some(ValueType::NullValue(NullValue::NullValue as i32)),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocFields {
    pub(crate) field_mask: DocumentMask,
    pub(crate) fields: HashMap<String, firestore::Value>,
}

#[derive(Debug)]
pub struct InvalidSerializeTarget<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> InvalidSerializeTarget<T> {
    fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for InvalidSerializeTarget<T> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_invalid_ser_traits {
    ($(($trait:ty, $($fn_name:ident),* $(,)?)),* $(,)?) => {
        $(
            impl<T> $trait for InvalidSerializeTarget<T> {
                type Ok = T;
                type Error = ConvertError;

                $(
                    fn $fn_name<V>(&mut self, _value: &V) -> Result<(), Self::Error>
                    where
                        V: Serialize + ?Sized
                    {
                        Err(ConvertError::ser("invalid serialization target"))
                    }
                )*

                fn end(self) -> Result<Self::Ok, Self::Error> {
                    Err(ConvertError::ser("invalid serialization target"))
                }
            }
        )*
    };
}

impl_invalid_ser_traits! {
    (serde::ser::SerializeSeq, serialize_element),
    (serde::ser::SerializeTuple, serialize_element),
    (serde::ser::SerializeTupleStruct, serialize_field),
    (serde::ser::SerializeTupleVariant, serialize_field),
    (serde::ser::SerializeMap, serialize_key, serialize_value),
}

impl<T> serde::ser::SerializeStruct for InvalidSerializeTarget<T> {
    type Ok = T;
    type Error = ConvertError;

    fn serialize_field<V>(&mut self, _key: &'static str, _value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        Err(ConvertError::ser("invalid serialization target"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::ser("invalid serialization target"))
    }
}

impl<T> serde::ser::SerializeStructVariant for InvalidSerializeTarget<T> {
    type Ok = T;
    type Error = ConvertError;

    fn serialize_field<V>(&mut self, _key: &'static str, _value: &V) -> Result<(), Self::Error>
    where
        V: Serialize + ?Sized,
    {
        Err(ConvertError::ser("invalid serialization target"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::ser("invalid serialization target"))
    }
}

/// Escapes invalid characters in document field paths. Assumes 'path' __does__ need to be escaped.
fn escape_component_into(parent: &mut String, path: &str) {
    if !parent.is_empty() && !parent.ends_with('.') {
        parent.push('.');
    }

    parent.reserve(path.len() + 2);

    parent.push('`');

    for ch in path.chars() {
        if ch == '`' {
            parent.push_str("\\`");
        } else {
            parent.push(ch);
        }
    }

    parent.push('`');
}

pub fn escape_field_path_into<I>(parts: I, dst: &mut String)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    dst.clear();

    for part in parts {
        Key::Map(part.as_ref()).escape_into_parent(dst);
    }
}

pub(crate) fn escape_field_path(s: &str) -> String {
    let mut dst = String::with_capacity(s.len() + 6);
    escape_field_path_into(s.split_terminator('.'), &mut dst);
    dst
}

fn component_needs_escaping(s: &str) -> bool {
    // macro to check for invalid characters.
    macro_rules! is_invalid_char {
        (NO_NUMBERS $ch:expr) => {{
            !matches!($ch, '_' | 'a'..='z' | 'A'..='Z')
        }};
        ($ch:expr) => {{
            !matches!($ch, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9')
        }};
    }

    // quick check to make sure the leading char is invalid. leading chars cant be numbers,
    // even though numbers are valid elsewhere.
    if s.starts_with(|ch: char| is_invalid_char!(NO_NUMBERS ch)) {
        return true;
    }

    // then verify all characters in the path are a-z, 0-9 or _.
    s.chars().any(|ch| is_invalid_char!(ch))
}

enum Key<'a> {
    #[allow(unused)]
    Index(usize),
    Map(&'a str),
}

impl<'a> Key<'a> {
    fn escape_key(&self) -> Cow<'a, str> {
        match self {
            Self::Map(m) => {
                if component_needs_escaping(m) {
                    let mut s = String::with_capacity(m.len() + 2);
                    escape_component_into(&mut s, m);
                    Cow::Owned(s)
                } else {
                    Cow::Borrowed(m)
                }
            }
            Self::Index(i) => format!("`{i}`").into(),
        }
    }

    fn escape_into_parent(self, parent: &mut String) {
        if !parent.is_empty() && !parent.ends_with('.') {
            parent.push('.');
        }

        match self {
            Self::Map(m) => {
                if component_needs_escaping(m) {
                    escape_component_into(parent, m);
                } else {
                    parent.push_str(m);
                }
            }
            Self::Index(i) => {
                std::fmt::Write::write_fmt(parent, format_args!("`{i}`"))
                    .expect("string formatting should never fail");
            }
        }
    }
}

fn build_mask(fields: &HashMap<String, firestore::Value>) -> Vec<String> {
    fn build_field_mask(
        parent: Option<Cow<'_, str>>,
        field_paths: &mut Vec<String>,
        key: Key<'_>,
        value: &firestore::Value,
    ) {
        let value_type = match value.value_type.as_ref() {
            Some(t) => t,
            None => return,
        };

        fn build_nested_parent<'k>(parent: Option<Cow<'_, str>>, key: Key<'k>) -> Cow<'k, str> {
            match parent {
                Some(parent) => {
                    let mut base = parent.into_owned();
                    key.escape_into_parent(&mut base);
                    Cow::Owned(base)
                }
                None => key.escape_key(),
            }
        }

        match value_type {
            ValueType::MapValue(MapValue { fields }) => {
                let nested_parent = build_nested_parent(parent, key);

                for (key, value) in fields.iter() {
                    build_field_mask(
                        Some(Cow::Borrowed(&*nested_parent)),
                        field_paths,
                        Key::Map(key),
                        value,
                    );
                }
            }
            /*
            ValueType::ArrayValue(ArrayValue { values }) => {
                let nested_parent = build_nested_parent(parent, key);

                for (idx, value) in values.iter().enumerate() {
                    build_field_mask(
                        Some(Cow::Borrowed(&*nested_parent)),
                        field_paths,
                        Key::Index(idx),
                        value,
                    );
                }
            }
            */
            _ => {
                let field_path = match parent {
                    Some(parent) => {
                        let mut dst = parent.into_owned();
                        key.escape_into_parent(&mut dst);
                        dst
                    }
                    None => key.escape_key().into_owned(),
                };

                field_paths.push(field_path);
            }
        }
    }

    let mut dst = Vec::with_capacity(fields.len());

    for (key, value) in fields.iter() {
        build_field_mask(None, &mut dst, Key::Map(key), value);
    }

    dst
}

#[test]
fn test_escape_field_path() {
    const TEST_CASES: &[(&str, &str)] = &[
        ("path.to.field", "path.to.field"),
        ("path.with space.field", "path.`with space`.field"),
        ("path.with`tick.field", "path.`with\\`tick`.field"),
        (
            "path.with`tick.with space.tick`2.field",
            "path.`with\\`tick`.`with space`.`tick\\`2`.field",
        ),
        (
            "path.multiple ` things.field",
            "path.`multiple \\` things`.field",
        ),
        ("path.0leading_num.field", "path.`0leading_num`.field"),
        ("path.0.field", "path.`0`.field"),
    ];

    for (unescaped, expected) in TEST_CASES {
        let escaped = escape_field_path(unescaped);
        assert_eq!(escaped.as_str(), *expected);
    }
}

#[test]
fn test_component_escape_test() {
    const TEST_CASES: &[(&str, bool)] = &[
        ("valid", false),
        ("has space", true),
        ("0leading_num", true),
        ("long_field_name_but_is_valid", false),
    ];

    for (comp, needs_escaping) in TEST_CASES {
        assert_eq!(*needs_escaping, component_needs_escaping(comp), "{comp}");
    }
}
