use std::marker::PhantomData;

use serde::de::{self, IntoDeserializer};

use crate::table::{TableFieldSchema, TableSchema};

pub(super) struct RowVisitor<'a, S, Row> {
    pub(super) schema: &'a TableSchema<S>,
    pub(super) _marker: PhantomData<fn(Row)>,
}

impl<'de, S, Row> de::DeserializeSeed<'de> for &mut RowVisitor<'_, S, Row>
where
    Row: serde::Deserialize<'de>,
    S: AsRef<str>,
{
    type Value = Row;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, S, Row> de::Visitor<'de> for &mut RowVisitor<'_, S, Row>
where
    Row: serde::Deserialize<'de>,
    S: AsRef<str>,
{
    type Value = Row;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of fields")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        #[derive(serde::Deserialize)]
        enum Field {
            #[serde(rename = "f")]
            F,
            #[serde(other)]
            Other,
        }

        let mut row = None;

        while let Some(field) = map.next_key()? {
            match field {
                Field::F if row.is_some() => return Err(de::Error::duplicate_field("f")),
                Field::F => {
                    row = Some(map.next_value_seed(RowFieldsVisitor {
                        schema: self.schema,
                        _marker: PhantomData,
                    })?)
                }
                Field::Other => {
                    map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        row.ok_or_else(|| de::Error::missing_field("f"))
    }
}

struct RowFieldsVisitor<'a, S, Row> {
    schema: &'a TableSchema<S>,
    _marker: PhantomData<fn(Row)>,
}

impl<'de, S, Row> de::DeserializeSeed<'de> for RowFieldsVisitor<'_, S, Row>
where
    Row: serde::Deserialize<'de>,
    S: AsRef<str>,
{
    type Value = Row;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}
impl<'de, S, Row> de::Visitor<'de> for RowFieldsVisitor<'_, S, Row>
where
    Row: serde::Deserialize<'de>,
    S: AsRef<str>,
{
    type Value = Row;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a single row (as an array of fields)")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        Row::deserialize(RowDeserializer {
            columns: self.schema.fields.iter(),
            seq_access: seq,
            current: None,
            _marker: PhantomData,
        })
    }
}

struct RowDeserializer<'a, 'de, S, A: de::SeqAccess<'de>> {
    columns: std::slice::Iter<'a, TableFieldSchema<S>>,
    current: Option<&'a TableFieldSchema<S>>,
    seq_access: A,
    _marker: PhantomData<fn(&'de ())>,
}

impl<'de, S, A> serde::Deserializer<'de> for RowDeserializer<'_, 'de, S, A>
where
    A: de::SeqAccess<'de>,
    S: AsRef<str>,
{
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, S, A> de::MapAccess<'de> for RowDeserializer<'_, 'de, S, A>
where
    A: de::SeqAccess<'de>,
    S: AsRef<str>,
{
    type Error = A::Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.columns.len())
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some(column) = self.columns.next() else {
            return Ok(None);
        };

        self.current = Some(column);

        seed.deserialize(column.name.as_ref().into_deserializer())
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let column = self
            .current
            .take()
            .expect("next_value_seed called out of order/after being emptied");

        self.seq_access
            .next_element_seed(super::value::ValueMapSeed::new(column, seed))?
            .ok_or_else(|| de::Error::custom("found fewer fields than expecting"))
    }
}
