use std::io::{Cursor, Read, Seek};
use std::marker::PhantomData;
use std::sync::Arc;

use apache_avro::Schema;
use apache_avro::types::Value;
use serde::de::{self, IntoDeserializer};
use serde::forward_to_deserialize_any;

use crate::error::Error;

fn is_null_union(schema: &Schema) -> bool {
    match schema {
        Schema::Map(inner) => is_null_union(&*inner.types),
        Schema::Union(u) => {
            if u.variants().len() != 2 {
                false
            } else {
                u.variants()
                    .iter()
                    .any(|union_schema| Schema::Null.eq(union_schema))
            }
        }
        _ => false,
    }
}

pub fn deserialize<'de, O>(value: Value, schema: &Schema) -> Result<O, Error>
where
    O: de::DeserializeOwned,
{
    let de = AvroDeserializer::new(value, schema);
    let wrapped_de = path_aware_serde::Deserializer::new(de);

    O::deserialize(wrapped_de).map_err(Error::from)
}

pub fn deserialize_seed<'a, 'de, S>(
    seed: S,
    value: Value,
    schema: &'a Schema,
) -> Result<S::Value, Error>
where
    S: de::DeserializeSeed<'de>,
    'a: 'de,
{
    let de = AvroDeserializer::new(value, schema);
    let wrapped_de = path_aware_serde::Deserializer::new(de);

    seed.deserialize(wrapped_de).map_err(Error::from)
}

#[derive(Debug)]
pub struct RowsDeserializer<R = Cursor<Vec<u8>>> {
    reader: R,
    schema: Arc<Schema>,
    row_count: usize,
    len: usize,
}

pub struct RowIter<S, R = Cursor<Vec<u8>>>
where
    for<'de> S: de::DeserializeSeed<'de> + Clone,
{
    de: RowsDeserializer<R>,
    rows_deserialized: usize,
    seed: S,
}

impl<S, R, O> Iterator for RowIter<S, R>
where
    R: Read + Seek,
    for<'de> S: de::DeserializeSeed<'de, Value = O> + Clone,
{
    type Item = Result<O, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.de.deserialize_value(self.seed.clone()).transpose()?;

        self.rows_deserialized += 1;

        Some(item)
    }
}

impl<S, R, O> ExactSizeIterator for RowIter<S, R>
where
    R: Read + Seek,
    for<'de> S: de::DeserializeSeed<'de, Value = O> + Clone,
{
    fn len(&self) -> usize {
        self.de.row_count - self.rows_deserialized
    }
}

impl<R> RowsDeserializer<R>
where
    R: Read + Seek,
{
    pub(super) fn new(mut reader: R, schema: Arc<Schema>, row_count: usize) -> Result<Self, Error> {
        let len = reader.stream_len()? as usize;
        Ok(Self {
            reader,
            schema,
            row_count,
            len,
        })
    }

    pub fn consume<O>(self) -> Result<Vec<O>, Error>
    where
        for<'de> O: de::Deserialize<'de>,
    {
        self.consume_with_seed(&PhantomData)
    }

    pub fn consume_with_seed<'de, S>(
        mut self,
        seed: &S,
    ) -> Result<Vec<<S as de::DeserializeSeed<'de>>::Value>, Error>
    where
        S: de::DeserializeSeed<'de> + Clone,
    {
        let mut batch = Vec::with_capacity(self.row_count);

        while let Some(row) = self.deserialize_value(seed.clone())? {
            batch.push(row);
        }

        Ok(batch)
    }

    fn remaining_bytes(&mut self) -> Result<usize, Error> {
        let pos = self.reader.stream_position()? as usize;

        self.len
            .checked_sub(pos)
            .ok_or_else(|| Error::Misc("stream byte position > stream len".to_owned()))
    }

    fn is_stream_empty(&mut self) -> Result<bool, Error> {
        self.remaining_bytes().map(|rem| rem == 0)
    }

    fn take_value(&mut self) -> Option<Result<Value, Error>> {
        match self.is_stream_empty() {
            Ok(true) => return None,
            Err(err) => return Some(Err(err)),
            _ => (),
        }

        let result = apache_avro::from_avro_datum(&*self.schema, &mut self.reader, None)
            .map_err(Error::from);

        Some(result)
    }

    pub(super) fn deserialize_value<'de, S>(&mut self, seed: S) -> Result<Option<S::Value>, Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        if self.is_stream_empty()? {
            return Ok(None);
        }

        let value = apache_avro::from_avro_datum(&*self.schema, &mut self.reader, None)?;

        let de = AvroDeserializer::new(value, &*self.schema);
        let wrapped_de = path_aware_serde::Deserializer::new(de);

        let item = seed.deserialize(wrapped_de)?;

        Ok(Some(item))
    }

    pub fn row_iter_with_seed<S>(self, seed: S) -> RowIter<S, R>
    where
        for<'de> S: de::DeserializeSeed<'de> + Clone,
    {
        RowIter {
            de: self,
            rows_deserialized: 0,
            seed,
        }
    }

    pub fn row_iter<O>(self) -> RowIter<PhantomData<O>, R>
    where
        for<'de> O: de::Deserialize<'de>,
    {
        RowIter {
            de: self,
            rows_deserialized: 0,
            seed: PhantomData,
        }
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }
}

impl<'de, R> de::Deserializer<'de> for RowsDeserializer<R>
where
    R: Read + Seek,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, R> de::SeqAccess<'de> for RowsDeserializer<R>
where
    R: Read + Seek,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = match self.take_value() {
            Some(Ok(val)) => val,
            Some(Err(err)) => return Err(err),
            None => return Ok(None),
        };

        let de = AvroDeserializer::new(value, &*self.schema);
        let wrapped_de = path_aware_serde::Deserializer::new(de);

        let item = seed.deserialize(wrapped_de)?;

        Ok(Some(item))
    }
}

#[derive(Debug)]
pub struct AvroDeserializer<'a> {
    value: Value,
    schema: &'a Schema,
}

trait Unexpected {
    fn to_unexpected(&self) -> de::Unexpected<'_>;
}

impl Unexpected for Value {
    fn to_unexpected(&self) -> de::Unexpected<'_> {
        match self {
            Value::Null => de::Unexpected::Other("null"),
            Value::Boolean(boolean) => de::Unexpected::Bool(*boolean),
            Value::Int(int) => de::Unexpected::Signed(*int as i64),
            Value::Long(long) => de::Unexpected::Signed(*long),
            Value::Float(float) => de::Unexpected::Float(*float as f64),
            Value::Double(double) => de::Unexpected::Float(*double),
            Value::Bytes(bytes) => de::Unexpected::Bytes(bytes.as_slice()),
            Value::String(string) => de::Unexpected::Str(string.as_str()),
            Value::Fixed(_, _) => de::Unexpected::Other("avro fixed value"),
            Value::Enum(_, string) => de::Unexpected::Other(string.as_str()),
            Value::Union(_, inner) => inner.to_unexpected(),
            Value::Array(_) => de::Unexpected::Seq,
            Value::Map(_) => de::Unexpected::Map,
            Value::Record(_) => de::Unexpected::Other("avro record"),
            Value::Date(avro_date) => de::Unexpected::Signed(*avro_date as i64),
            Value::Decimal(_) => de::Unexpected::Other("avro decimal value"),
            Value::TimeMillis(avro_date) => de::Unexpected::Signed(*avro_date as i64),
            Value::TimeMicros(avro_date) => de::Unexpected::Signed(*avro_date),
            Value::TimestampMillis(avro_date) => de::Unexpected::Signed(*avro_date),
            Value::TimestampMicros(avro_date) => de::Unexpected::Signed(*avro_date),
            Value::TimestampNanos(avro_nanos) => de::Unexpected::Signed(*avro_nanos),
            Value::Duration(_) => de::Unexpected::Other("avro duration"),
            Value::Uuid(_) => de::Unexpected::Other("uuid"),
            Value::BigDecimal(_) => de::Unexpected::Other("avro big decimal"),
            Value::LocalTimestampMicros(_) => de::Unexpected::Other("avro local timestamp micros"),
            Value::LocalTimestampMillis(_) => de::Unexpected::Other("avro local timestamp millis"),
            Value::LocalTimestampNanos(_) => de::Unexpected::Other("avro local timestamp nanos"),
        }
    }
}

impl<'a> AvroDeserializer<'a> {
    pub(crate) fn new(value: Value, schema: &'a Schema) -> Self {
        Self { value, schema }
    }

    pub(crate) fn flatten_union(&mut self) {
        if let Value::Union(_, _) = &self.value {
            let value = std::mem::replace(&mut self.value, Value::Null);

            self.value = match value {
                Value::Union(_, boxed) => Box::into_inner(boxed),
                _ => unreachable!(),
            };
        }
    }

    fn get_int(mut self) -> Result<i64, Value> {
        self.flatten_union();

        match self.value {
            Value::Int(int) => Ok(int as i64),
            Value::Long(long) => Ok(long),
            Value::Float(float) => Ok(float as i64),
            Value::Double(double) => Ok(double as i64),
            Value::TimeMillis(millis) => Ok(millis as i64 / 1000),
            Value::TimestampMillis(millis) => Ok(millis as i64 / 1000),
            Value::TimeMicros(micros) | Value::TimestampMicros(micros) => {
                Ok(micros as i64 / 1_000_000)
            }
            /*
            Value::Decimal(decimal) => {
                todo!()
            },
            Value::Duration(duration) => {
                let months = u32::from_le_bytes(*duration.months().as_ref());
                let days = u32::from_le_bytes(*duration.days().as_ref());
                let millis = u32::from_le_bytes(*duration.millis().as_ref());

                let mut seconds = millis as f64 / 1e3;
                seconds += days as f64 * 24.0 * 3600.0;
            },
            */
            val => Err(val),
        }
    }

    fn get_float(mut self) -> Result<f64, Value> {
        self.flatten_union();

        match self.value {
            Value::Int(int) => Ok(int as f64),
            Value::Long(long) => Ok(long as f64),
            Value::Float(float) => Ok(float as f64),
            Value::Double(double) => Ok(double),
            Value::TimeMillis(millis) => Ok(millis as f64 / 1e3),
            Value::TimestampMillis(millis) => Ok(millis as f64 / 1e3),
            Value::TimeMicros(micros) | Value::TimestampMicros(micros) => Ok(micros as f64 / 1e6),
            /*
            Value::Decimal(decimal) => {
                todo!()
            },
            Value::Duration(duration) => {
                let months = u32::from_le_bytes(*duration.months().as_ref());
                let days = u32::from_le_bytes(*duration.days().as_ref());
                let millis = u32::from_le_bytes(*duration.millis().as_ref());

                let mut seconds = millis as f64 / 1e3;
                seconds += days as f64 * 24.0 * 3600.0;
            },
            */
            val => Err(val),
        }
    }
}

impl<'a, 'de> de::Deserializer<'de> for AvroDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Boolean(boolean) => visitor.visit_bool(boolean),
            Value::Int(int) => visitor.visit_i32(int),
            Value::Long(long) => visitor.visit_i64(long),
            Value::Float(float) => visitor.visit_f32(float),
            Value::Double(double) => visitor.visit_f64(double),
            Value::Bytes(bytes) => visitor.visit_byte_buf(bytes),
            Value::String(string) => visitor.visit_string(string),
            Value::Fixed(f, _) => todo!("Value::Fixed: {f}"),
            Value::Enum(f, s) => todo!("Value::Enum: {f} - {s}"),
            Value::Union(_, boxed) if is_null_union(self.schema) => {
                Self::new(Box::into_inner(boxed), self.schema).deserialize_option(visitor)
            }
            Value::Union(_, boxed) => {
                Self::new(Box::into_inner(boxed), self.schema).deserialize_any(visitor)
            }
            Value::Array(array) => visitor.visit_seq(SeqAccess::new(array, self.schema)),
            Value::Map(map) => visitor.visit_map(MapAccess::new(map, self.schema)),
            Value::Record(record) => visitor.visit_map(MapAccess::new(record, self.schema)),
            Value::Date(d) => todo!("Value::Date: {d}"),
            Value::Decimal(d) => todo!("Value::Decimal: {d:#?}"),
            Value::TimeMillis(d) => todo!("Value::TimeMillis: {d}"),
            Value::TimeMicros(d) => todo!("Value::Fixed: {d}"),
            Value::TimestampMillis(ts_millis) => {
                let ts = timestamp::Timestamp::from_millis_checked(ts_millis)
                    .map_err(|e| Error::InvalidTimestamp(e.into()))?;

                ts.into_deserializer()
                    .deserialize_any(visitor)
                    .map_err(crate::Error::from)
            }
            Value::TimestampMicros(ts_micros) => {
                let ts = timestamp::Timestamp::from_micros_checked(ts_micros)
                    .map_err(|e| Error::InvalidTimestamp(e.into()))?;

                ts.into_deserializer()
                    .deserialize_any(visitor)
                    .map_err(crate::Error::from)
            }
            Value::TimestampNanos(nanos) => timestamp::Timestamp::from_nanos(nanos)
                .into_deserializer()
                .deserialize_any(visitor)
                .map_err(crate::Error::from),
            x @ (Value::Duration(_)
            | Value::LocalTimestampMillis(_)
            | Value::LocalTimestampMicros(_)
            | Value::LocalTimestampNanos(_)
            | Value::BigDecimal(_)) => todo!("avro value type {x:#?}"),
            #[cfg(integer128)]
            Value::Uuid(uuid) => visitor.visit_u128(uuid.as_u128()),
            #[cfg(not(integer128))]
            Value::Uuid(uuid) => visitor.visit_string(uuid.to_string()),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let int = match self.get_int() {
            Ok(int) => int,
            Err(val) => {
                return Err(de::Error::invalid_type(
                    val.to_unexpected(),
                    &"an unsigned int",
                ));
            }
        };

        let uint: u64 = int
            .try_into()
            .map_err(|err: <i64 as TryInto<u64>>::Error| Error::Misc(err.to_string()))?;

        visitor.visit_u64(uint)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.get_int() {
            Ok(int) => visitor.visit_i64(int),
            Err(val) => Err(de::Error::invalid_type(val.to_unexpected(), &"i64")),
        }
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();
        match self.value {
            Value::Bytes(bytes) => match bytes.first() {
                Some(byte) => visitor.visit_char(*byte as char),
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Bytes(&bytes),
                    &"a char",
                )),
            },
            Value::String(string) => match string.chars().next() {
                Some(c) => visitor.visit_char(c),
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Str(&string),
                    &"a char",
                )),
            },
            val => Err(de::Error::invalid_type(val.to_unexpected(), &"a char")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.get_float() {
            Ok(float) => visitor.visit_f64(float),
            Err(val) => Err(de::Error::invalid_type(
                val.to_unexpected(),
                &"a floating point value",
            )),
        }
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        match self.value {
            Value::Boolean(boolean) => visitor.visit_bool(boolean),
            val => Err(de::Error::invalid_type(val.to_unexpected(), &"boolean")),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();
        match self.value {
            Value::Null => visitor.visit_unit(),
            val => Err(de::Error::invalid_type(val.to_unexpected(), &"null")),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        match self.value {
            Value::Map(map) => visitor.visit_map(MapAccess::new(map.into_iter(), self.schema)),
            Value::Record(record) => {
                visitor.visit_map(MapAccess::new(record.into_iter(), self.schema))
            }
            val => Err(de::Error::invalid_type(
                val.to_unexpected(),
                &"a map of string/value pairs",
            )),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        type DecimalErr = <apache_avro::Decimal as TryInto<Vec<u8>>>::Error;

        match self.value {
            Value::Bytes(bytes) => visitor.visit_byte_buf(bytes),
            Value::String(string) => visitor.visit_byte_buf(string.into_bytes()),
            Value::Decimal(decimal) => {
                let bytes: Vec<u8> = decimal
                    .try_into()
                    .map_err(|err: DecimalErr| Error::Misc(err.to_string()))?;
                visitor.visit_byte_buf(bytes)
            }
            val => Err(de::Error::invalid_type(
                val.to_unexpected(),
                &"buffer of bytes",
            )),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        match self.value {
            Value::String(string) => visitor.visit_string(string),
            Value::Bytes(bytes) => {
                let string =
                    String::from_utf8(bytes).map_err(|err| Error::Misc(err.to_string()))?;
                visitor.visit_string(string)
            }
            Value::Uuid(uuid) => visitor.visit_string(uuid.to_string()),
            val => Err(de::Error::invalid_type(val.to_unexpected(), &"string")),
        }
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        match self.value {
            Value::Array(array) => visitor.visit_seq(SeqAccess::new(array, self.schema)),
            Value::Map(map) => visitor.visit_seq(SeqAccess::new(map.into_values(), self.schema)),
            Value::Record(record) => {
                let value_iter = record.into_iter().map(|(_, v)| v);
                visitor.visit_seq(SeqAccess::new(value_iter, self.schema))
            }
            val => Err(de::Error::invalid_type(
                val.to_unexpected(),
                &"a sequence of avro values",
            )),
        }
    }

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        match &self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.flatten_union();

        match self.value {
            Value::Array(values) => {
                let pair_iter = fields.into_iter().copied().zip(values.into_iter());
                visitor.visit_map(MapAccess::new(pair_iter, self.schema))
            }
            Value::Map(map) => visitor.visit_map(MapAccess::new(map, self.schema)),
            Value::Record(record) => visitor.visit_map(MapAccess::new(record, self.schema)),
            val => Err(de::Error::invalid_type(val.to_unexpected(), &name)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

pub struct SeqAccess<'a, I> {
    value_iter: I,
    schema: &'a Schema,
}

impl<'a, I> SeqAccess<'a, I> {
    fn new<C>(value_iter: C, schema: &'a Schema) -> Self
    where
        C: IntoIterator<IntoIter = I>,
    {
        Self {
            value_iter: value_iter.into_iter(),
            schema,
        }
    }
}

impl<'a, 'de, I> de::SeqAccess<'de> for SeqAccess<'a, I>
where
    I: Iterator<Item = Value> + ExactSizeIterator,
{
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.value_iter.len())
    }

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = match self.value_iter.next() {
            Some(val) => val,
            None => return Ok(None),
        };

        seed.deserialize(AvroDeserializer::new(value, self.schema))
            .map(Some)
    }
}

pub struct MapAccess<'a, I> {
    kvp_iter: I,
    schema: &'a Schema,
    next_value: Option<Value>,
}

impl<'a, I> MapAccess<'a, I> {
    fn new<C>(kvp_iter: C, schema: &'a Schema) -> Self
    where
        C: IntoIterator<IntoIter = I>,
    {
        Self {
            kvp_iter: kvp_iter.into_iter(),
            schema,
            next_value: None,
        }
    }
}

impl<'a, 'de, I, S> de::MapAccess<'de> for MapAccess<'a, I>
where
    I: Iterator<Item = (S, Value)> + ExactSizeIterator,
    S: Into<String>,
{
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        Some(self.kvp_iter.len())
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let (key, value) = match self.kvp_iter.next() {
            Some((k, v)) => (k, v),
            None => return Ok(None),
        };

        if self.next_value.replace(value).is_some() {
            error!("value skipped in MapAccess");
        }

        seed.deserialize(key.into().into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self
            .next_value
            .take()
            .ok_or_else(|| Error::Misc("next_value_seed has no value to yield".to_owned()))?;

        seed.deserialize(AvroDeserializer::new(value, self.schema))
    }
}

impl<'a, 'de> de::EnumAccess<'de> for AvroDeserializer<'a> {
    type Variant = VariantAccess<'de>;
    type Error = Error;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self)?;

        Ok((value, VariantAccess {
            _marker: PhantomData,
        }))
    }
}

pub struct VariantAccess<'de> {
    _marker: PhantomData<&'de ()>,
}

impl<'de> de::VariantAccess<'de> for VariantAccess<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        todo!("newtype_variant_seed")
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!("tuple_variant")
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!("struct_variant: {fields:#?}")
    }
}

/*

#[derive(Clone, Copy, Debug, Builder)]
#[builder(build_fn(skip))]
pub struct AvroDeser<'de> {
    #[builder(setter(skip))]
    value: &'de Value,
}

#[derive(Debug)]
pub(crate) struct AvroError(anyhow::Error);

impl From<anyhow::Error> for AvroError {
    fn from(any: anyhow::Error) -> Self {
        Self(any)
    }
}

impl ops::Deref for AvroError {
    type Target = anyhow::Error;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for AvroError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

use std::fmt;

impl fmt::Display for AvroError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:#?}", self)
    }
}

impl std::error::Error for AvroError {}

impl de::Error for AvroError {
    fn custom<T>(item: T) -> Self
    where
        T: fmt::Display
    {
        de::Error::custom(item)
    }
}


pub struct AvroArrayDeser<'de> {
    iter: std::slice::Iter<'de, Value>,
}

pub struct AvroMapDeser<'de> {
    keys: hash_map::Keys<'de, String, Value>,
    values: hash_map::Values<'de, String, Value>,
}

pub struct AvroRecDeser<'de> {
    field_iter: std::slice::Iter<'de, (String, Value)>,
    value: Option<&'de Value>,
}



fn avro_duration_to_secs(dur: avro_rs::Duration) -> f64 {
    let buff: [u8; 12] = dur.into();

    // SAFETY: Since we know the initial array is 12 elements, we can split
    // it unchecked into 3 chunks of 4. Similarly, since we know there's going
    // to be 3 chunks, we can index unchecked to the 0th, 1st and 2nd indicies.
    let split: &[[u8; 4]] = unsafe { buff.as_chunks_unchecked() };

    let months = u32::from_le_bytes(unsafe { *split.get_unchecked(0) });
    let days = u32::from_le_bytes(unsafe { *split.get_unchecked(1) });
    let millis = u32::from_le_bytes(unsafe { *split.get_unchecked(2) });


    let whole_seconds = (months * SECONDS_PER_MONTH) + (days * SECONDS_PER_DAY);

    (whole_seconds as f64) + (millis as f64 / 1000.0)
}

macro_rules! impl_avro_visitor_fn {
    ($fn_name:ident) => {
        fn $fn_name<'de, V>(value: &'de Value, visitor: V) -> Result<V::Value, AvroError>
        where
            V: de::Visitor<'de>
        {
            match value {
                Value::Null => visitor.visit_none(),
                Value::Boolean(b) => visitor.visit_bool(*b),
                Value::Int(i)
                    | Value::Date(i)
                    | Value::TimeMillis(i) => visitor.visit_i32(*i),
                Value::Long(l)
                    | Value::TimeMicros(l)
                    | Value::TimestampMillis(l)
                    | Value::TimestampMicros(l) => visitor.visit_i64(*l),
                Value::Float(fl) => visitor.visit_f32(*fl),
                Value::Double(db) => visitor.visit_f64(*db),
                Value::Union(un) => visitor.visit_some(&AvroDeser::new(&*un)),
                Value::String(string) => visitor.visit_borrowed_str(string),
                Value::Uuid(uuid) => visitor.visit_string(uuid.to_string()),
                Value::Fixed(_, bytes)
                    | Value::Bytes(bytes) => visitor.visit_borrowed_bytes(bytes),
                Value::Duration(dur) => visitor.visit_f64(avro_duration_to_secs(*dur)),
                Value::Array(array) => visitor.visit_seq(&mut AvroArrayDeser::new(array)),
                Value::Map(map) => visitor.visit_map(&mut AvroMapDeser::new(map)),
                Value::Record(fields) => visitor.visit_map(&mut AvroRecDeser::new(fields)),
                misc_val => {
                    let kind = avro_rs::types::ValueKind::from(misc_val);
                    unimplemented!("{:?} deserialization not implemented", kind)
                }
            }
        }
    };
}

impl_avro_visitor_fn!(visit_avro_field);


impl<'de> AvroDeser<'de> {
    pub(crate) fn new(value: &'de Value) -> Self {
        Self { value }
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a AvroDeser<'de> {
    type Error = AvroError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
    {
        visit_avro_field(self.value, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct tuple enum identifier ignored_any
    }
}

impl<'de> AvroArrayDeser<'de> {
    fn new<I>(collec: I) -> Self
    where
        I: IntoIterator
    {
        collec.into_iter()
    }
}

impl<'de> AvroMapDeser<'de> {
    fn new<K, V, M>(map: M) -> Self
    where
        K: de::Deserialize<'de>,
        V: de::Deserialize<'de>,
        M: Into<hash_map::HashMap<K, V>>
    {
        let hash = hash_map::HashMap::from(map);

        Self { keys: hash.keys(), values: hash.values() }
    }
}

impl<'de> AvroRecDeser<'de> {
    fn new(iter: ) -> Self {
        Self { value: None, field_iter }
    }
}


impl<'de> de::SeqAccess<'de> for AvroArrayDeser<'de> {
    type Error = AvroError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(&AvroDeser::new(value)).map(Some),
            None => Ok(None)
        }
    }
}

impl<'de> de::MapAccess<'de> for AvroMapDeser<'de> {
    type Error = AvroError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>
    {
        match self.keys.next() {
            Some(key) => seed.deserialize(KeyDeserializer { key }).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>
    {
        match self.values.next() {
            Some(value) => seed.deserialize(&AvroDeser::new(value)),
            None => Err(anyhow!("number of values > number of keys").into())
        }
    }
}



impl<'a, 'de> de::MapAccess<'de> for AvroRecDeser<'de> {
    type Error = AvroError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>
    {
        match self.field_iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(KeyDeserializer { key }).map(Some)
            },
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>
    {
        match self.value.take() {
            Some(value) => {
                seed.deserialize(&AvroDeser::new(value))
            },
            None => Err(anyhow!("number of values > number of keys").into())
        }

    }
}


#[derive(Clone)]
struct KeyDeserializer<'de> {
    key: &'de str,
}

impl<'de> de::Deserializer<'de> for KeyDeserializer<'de> {
    type Error = AvroError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.key)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct tuple enum identifier ignored_any
    }
}
*/
