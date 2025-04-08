use std::collections::HashMap;

use protos::firestore::value::ValueType;
use protos::firestore::{Document, Value};

pub(crate) fn encoded_document_size(document: &Document) -> usize {
    document_name_size(&document.name) + encoded_map_size(&document.fields) + 32
}

#[inline]
fn encoded_string_size(s: &str) -> usize {
    s.len() + 1
}

#[inline]
pub(crate) fn none_if_empty<T, Container: AsRef<[T]>>(container: Container) -> Option<Container> {
    if container.as_ref().is_empty() {
        None
    } else {
        Some(container)
    }
}

#[inline]
fn document_name_size(name: &str) -> usize {
    let trimmed_doc_path_opt = name.split_once("/documents/").map(|(_, doc_path)| doc_path);

    let Some(trimmed_doc_path) = trimmed_doc_path_opt else {
        return 0;
    };

    trimmed_doc_path
        .split('/')
        .map(encoded_string_size)
        .sum::<usize>()
        + 16
}

#[inline]
fn encoded_value_size(value: &Value) -> usize {
    match value.value_type.as_ref() {
        // treat None like null
        None => 1,
        Some(ValueType::ArrayValue(array)) => array.values.iter().map(encoded_value_size).sum(),
        Some(ValueType::BooleanValue(_)) => 1,
        Some(ValueType::BytesValue(bytes)) => bytes.len(),
        Some(ValueType::DoubleValue(_)) => 8,
        Some(ValueType::GeoPointValue(_)) => 16,
        Some(ValueType::IntegerValue(_)) => 8,
        Some(ValueType::MapValue(map)) => encoded_map_size(&map.fields),
        Some(ValueType::NullValue(_)) => 1,
        Some(ValueType::ReferenceValue(refer)) => document_name_size(refer),
        Some(ValueType::StringValue(string)) => encoded_string_size(&string),
        Some(ValueType::TimestampValue(_)) => 8,
    }
}

#[inline]
fn encoded_map_size<I>(map: &I) -> usize
where
    for<'a> &'a I: IntoIterator<Item = (&'a String, &'a Value)>,
{
    map.into_iter()
        .map(|(key, val)| {
            let key_size = encoded_string_size(&key);
            let value_size = encoded_value_size(&val);
            key_size + value_size
        })
        .sum()
}

pub(crate) fn extract_value<'a>(
    values: &'a HashMap<String, protos::firestore::Value>,
    field_path: &str,
) -> Option<crate::value::ValueRef<'a>> {
    let mut map = Some(values);
    let mut raw_value: Option<&protos::firestore::value::ValueType> = None;

    for field in field_path.split('.') {
        raw_value = map?.get(field)?.value_type.as_ref();

        map = match raw_value {
            Some(ValueType::MapValue(map)) => Some(&map.fields),
            _ => None,
        };
    }

    raw_value.map(crate::value::ValueRef::from_proto_type_ref)
}

/// compares resource paths, comparing on a path component basis
///
/// implemenation should be identical to:
/// https://github.com/googleapis/google-cloud-dotnet/blob/main/apis/Google.Cloud.Firestore/Google.Cloud.Firestore/PathComparer.cs
pub(crate) fn cmp_paths(a: &str, b: &str) -> std::cmp::Ordering {
    a.split('/').cmp(b.split('/'))
}
