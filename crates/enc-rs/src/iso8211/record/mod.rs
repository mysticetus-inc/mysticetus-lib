use std::collections::HashMap;
use std::io::Read;

use intern::InternedStr;
use intern::local::LocalInterner;

use crate::Iso8211Reader;

pub mod value;

pub use self::value::Value;
use super::descriptor::dd_field::{Field, FieldKind, Subfield};
use super::directory::DirectoryEntries;
use super::leader::DataLeader;
use super::{DataDescriptiveRecord, Iso8211Error};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LogicalRecord {
    leader: DataLeader,
    directory: DirectoryEntries,
    fields: Vec<HashMap<InternedStr, Value>>,
}

impl LogicalRecord {
    pub(crate) fn from_reader<R: Read>(
        reader: &mut Iso8211Reader<R>,
        ddr: &DataDescriptiveRecord,
        interner: &mut LocalInterner,
    ) -> Result<Self, Iso8211Error> {
        let init = reader.position();
        let leader = reader.parse_from_array::<DataLeader, 24>()?;

        let directory = DirectoryEntries::from_reader(reader, &leader, interner)?;

        let mut fields = Vec::new();

        let mut row = HashMap::with_capacity(directory.entries.len());

        for dir_entry in directory.entries.iter() {
            let value = ddr.parse_ddf_value(reader, interner, dir_entry)?;
            row.insert(dir_entry.tag.clone(), value);
        }

        if !row.is_empty() {
            fields.push(row);
        }

        let bytes_read = reader.position() - init;
        assert_eq!(bytes_read, leader.record_length);

        Ok(Self {
            leader,
            directory,
            fields,
        })
    }

    pub fn iter<'a>(&'a self, ddr: &'a DataDescriptiveRecord) -> FieldIter<'a> {
        FieldIter {
            fields: self.fields.iter(),
            current: None,
            ddr,
        }
    }
}

pub struct FieldIter<'a> {
    fields: std::slice::Iter<'a, HashMap<InternedStr, Value>>,
    current: Option<std::collections::hash_map::Iter<'a, InternedStr, Value>>,
    ddr: &'a DataDescriptiveRecord,
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = ResolvedField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (name, value) = loop {
                match self.current {
                    Some(ref mut inner) => {
                        if let Some(pair) = inner.next() {
                            break pair;
                        }
                    }
                    _ => (),
                }

                self.current = Some(self.fields.next()?.iter());
            };

            // skip the 0001 fields
            if name.find(|ch: char| ch.is_ascii_alphabetic()).is_none() {
                continue;
            }

            let ddf = match self.ddr.get_ddf(name) {
                Some(ddf) => ddf,
                None => {
                    println!("{name} not found in ddf");
                    continue;
                }
            };

            let field = ddf.kind();

            let values = match (field.kind(), value) {
                (FieldKind::Vector { subfields }, Value::Array(values)) => {
                    ResolvedValues::Array { subfields, values }
                }
                (FieldKind::Vector { .. }, _) => panic!("value not a vector {value:#?}"),
                (FieldKind::Elementary(subfield), _) => {
                    ResolvedValues::Elementary { subfield, value }
                }
                (FieldKind::Array { subfields }, Value::Array(values)) => {
                    ResolvedValues::Array { subfields, values }
                }
                (FieldKind::Array { .. }, _) => panic!("value not an array {value:#?}"),
                (kind, _value) => todo!("{kind:#?}"),
            };

            return Some(ResolvedField {
                name,
                field,
                values,
            });
        }
    }
}

#[derive(Debug)]
pub struct ResolvedField<'a> {
    name: &'a InternedStr,
    field: &'a Field,
    values: ResolvedValues<'a>,
}

impl serde::Serialize for ResolvedField<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(2))?;

        map.serialize_entry("name", self.name.as_str())?;
        // map.serialize_entry("field", &self.field)?;

        match self.values {
            ResolvedValues::Elementary { subfield, value } => {
                map.serialize_entry("values", &SerializeSubfieldValue { subfield, value })?;
            }
            ResolvedValues::Array { subfields, values } => {
                map.serialize_entry("values", &SerializeArrayAsMap { subfields, values })?;
            }
        }

        map.end()
    }
}

#[derive(Debug, serde::Serialize)]
pub enum ResolvedValues<'a> {
    Elementary {
        subfield: &'a Subfield<Option<InternedStr>>,
        value: &'a Value,
    },
    Array {
        subfields: &'a [Subfield<InternedStr>],
        values: &'a [Value],
    },
}

struct SerializeArrayAsMap<'a> {
    subfields: &'a [Subfield<InternedStr>],
    values: &'a [Value],
}

impl serde::Serialize for SerializeArrayAsMap<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map =
            serializer.serialize_map(Some(self.subfields.len().min(self.values.len())))?;

        for (subfield, value) in self.subfields.iter().zip(self.values.iter()) {
            map.serialize_entry(subfield.label.as_str(), &SerializeSubfieldValue {
                subfield,
                value,
            })?;
        }

        map.end()
    }
}

struct SerializeSubfieldValue<'a, T> {
    subfield: &'a Subfield<T>,
    value: &'a Value,
}

impl<T> serde::Serialize for SerializeSubfieldValue<'_, T>
where
    for<'a> &'a T: Into<Option<&'a InternedStr>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let label_opt: Option<&InternedStr> = (&self.subfield.label).into();
        let mut map = serializer.serialize_map(Some(2 + label_opt.is_some() as usize))?;

        if let Some(label) = label_opt {
            map.serialize_entry("label", label)?;
        }

        map.serialize_entry("format_control", &self.subfield.format_control)?;
        map.serialize_entry("value", &SerializeValue(self.value))?;
        map.end()
    }
}

struct SerializeValue<'a>(&'a Value);

impl serde::Serialize for SerializeValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Value::Int(i) => serializer.serialize_i64(*i as i64),
            Value::Uint(u) => serializer.serialize_u64(*u as u64),
            Value::String(s) => serializer.serialize_str(s.as_str()),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::Bytes(b) => serializer.serialize_bytes(b),
            Value::Complex(c) => c.serialize(serializer),
            Value::Array(a) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(a.len()))?;
                for elem in a {
                    seq.serialize_element(&SerializeValue(elem))?;
                }
                seq.end()
            }
        }
    }
}
