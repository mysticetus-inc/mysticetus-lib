//! Firestore document serializers for [`Serialize`]-able types.

use std::borrow::Cow;
use std::collections::HashMap;

use protos::firestore::document_transform::FieldTransform;
use protos::firestore::value::ValueType;
use protos::firestore::{self, DocumentMask, MapValue, Value};
use protos::protobuf::NullValue;

pub(crate) mod doc;
pub(crate) mod field_transform;
mod path;
mod value;

pub(crate) use value::ValueSerializer;

use crate::error::SerError;

pub(crate) trait WriteKind: 'static {
    const MERGE: bool;
}

pub(crate) enum Merge {}
pub(crate) enum Update {}

impl WriteKind for Merge {
    const MERGE: bool = true;
}

impl WriteKind for Update {
    const MERGE: bool = false;
}

pub(crate) fn serialize_write<W: WriteKind>(
    value: &(impl serde::Serialize + ?Sized),
) -> Result<(DocFields, Vec<FieldTransform>), SerError> {
    value.serialize(doc::WriteSerializer::<W>::NEW)
}

pub(crate) fn serialize_update<W: WriteKind>(
    value: &(impl serde::Serialize + ?Sized),
) -> Result<DocFields, SerError> {
    value.serialize(doc::UpdateSerializer::<W>::NEW)
}

pub(crate) fn serialize_value<W: WriteKind>(
    value: &(impl serde::Serialize + ?Sized),
) -> Result<firestore::value::ValueType, SerError> {
    match ValueSerializer::<W, std::convert::Infallible>::default().serialize(value)? {
        value::SerializedValueKind::Value(value) => Ok(value),
    }
}

#[inline]
const fn null_value() -> Value {
    Value {
        value_type: Some(ValueType::NullValue(NullValue::NullValue as i32)),
    }
}

trait MapSerializerKind<Arg = ()>:
    Sized
    + serde::ser::SerializeMap<Ok = Self::Output, Error = SerError>
    + serde::ser::SerializeStruct<Ok = Self::Output, Error = SerError>
    + serde::ser::SerializeStructVariant<Ok = Self::Output, Error = SerError>
{
    type Output;
    fn new_with_len(len: Option<usize>, arg: Arg) -> Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocFields {
    field_mask: Option<DocumentMask>,
    fields: HashMap<String, firestore::Value>,
}

impl DocFields {
    pub fn serialize_merge<T: serde::Serialize + ?Sized>(value: &T) -> crate::Result<Self> {
        serialize_update::<Merge>(value).map_err(crate::Error::from)
    }

    pub fn serialize<T: serde::Serialize + ?Sized>(value: &T) -> crate::Result<Self> {
        serialize_update::<Update>(value).map_err(crate::Error::from)
    }

    pub fn into_fields(self) -> HashMap<String, firestore::Value> {
        self.fields
    }

    pub fn into_fields_with_optional_mask(
        self,
        build_mask: bool,
    ) -> (HashMap<String, firestore::Value>, Option<DocumentMask>) {
        if build_mask {
            let (fields, mask) = self.into_fields_with_mask();
            (fields, Some(mask))
        } else {
            (self.into_fields(), None)
        }
    }

    pub fn into_fields_with_mask(self) -> (HashMap<String, firestore::Value>, DocumentMask) {
        let mask = match self.field_mask {
            Some(mask) if mask.field_paths.len() == self.fields.len() => mask,
            _ => DocumentMask {
                field_paths: build_mask(&self.fields),
            },
        };

        (self.fields, mask)
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

    let mut chars = s.chars();

    let Some(first) = chars.next() else {
        return false;
    };

    // quick check to make sure the leading char is invalid. leading chars cant be numbers,
    // even though numbers are valid elsewhere.
    if is_invalid_char!(NO_NUMBERS first) {
        return true;
    }

    // then verify the remaining characters in the path are a-z, 0-9 or _.
    chars.any(|ch| is_invalid_char!(ch))
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

pub(super) fn build_mask(fields: &HashMap<String, firestore::Value>) -> Vec<String> {
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
