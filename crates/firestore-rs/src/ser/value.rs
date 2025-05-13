//! A serializer to create [`firestore::Value`] from [`Serialize`]-able types.
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use protos::firestore::value::ValueType;
use protos::firestore::{self, ArrayValue, MapValue};
use protos::r#type::LatLng;
use serde::ser::{self, Serialize, Serializer};
use serde_helpers::key_capture::KeyCapture;

mod newtype;

use super::NullStrategy;
use crate::{ConvertError, Reference};

/// Handles serializing [`Serialize`]-able types into a [`firestore::Value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueSerializer<N = super::OmitNulls> {
    _marker: PhantomData<fn(N)>,
}

impl Default for ValueSerializer<super::OmitNulls> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<N> ValueSerializer<N>
where
    N: NullStrategy,
{
    pub(crate) const NEW: Self = Self {
        _marker: PhantomData,
    };

    pub(crate) fn seq<V>(self, raw_values: &[&V]) -> Result<firestore::Value, ConvertError>
    where
        V: Serialize,
    {
        let mut values = if N::OMIT {
            Vec::with_capacity(raw_values.len())
        } else {
            Vec::with_capacity(raw_values.len() / 2)
        };

        for result in raw_values.iter().map(|value| value.serialize(self)) {
            match result? {
                Some(val) => values.push(val),
                None => N::handle_null(|val| values.push(val)),
            }
        }

        Ok(firestore::Value {
            value_type: Some(ValueType::ArrayValue(ArrayValue { values })),
        })
    }
}

impl<N> Serializer for ValueSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    type SerializeSeq = ValueSeqSerializer<N>;
    type SerializeTuple = ValueSeqSerializer<N>;
    type SerializeTupleStruct = ValueSeqSerializer<N>;
    type SerializeTupleVariant = ValueSeqSerializer<N>;

    type SerializeMap = ValueMapSerializer<N>;
    type SerializeStruct = ValueMapSerializer<N>;
    type SerializeStructVariant = ValueMapSerializer<N>;

    fn serialize_bool(self, boolean: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::BooleanValue(boolean)),
        }))
    }

    fn serialize_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::BytesValue(bytes.to_vec().into())),
        }))
    }

    fn serialize_char(self, c: char) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::StringValue(String::from(c))),
        }))
    }

    fn serialize_f32(self, float: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(float as f64)
    }

    fn serialize_f64(self, float: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::DoubleValue(float)),
        }))
    }

    // Redirect all integer types to this master function
    fn serialize_i64(self, int: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::IntegerValue(int)),
        }))
    }

    fn serialize_i8(self, int: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_i16(self, int: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_i32(self, int: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u8(self, int: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u16(self, int: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u32(self, int: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(int as i64)
    }

    fn serialize_u64(self, int: u64) -> Result<Self::Ok, Self::Error> {
        match int.try_into().ok() {
            Some(casted_int) => self.serialize_i64(casted_int),
            None => self.serialize_str(&int.to_string()),
        }
    }

    serde::serde_if_integer128! {
        fn serialize_i128(self, int: i128) -> Result<Self::Ok, Self::Error> {
            match int.try_into().ok() {
                Some(casted_int) => self.serialize_i64(casted_int),
                None => self.serialize_str(&int.to_string()),
            }
        }

        fn serialize_u128(self, int: u128) -> Result<Self::Ok, Self::Error> {
            match int.try_into().ok() {
                Some(casted_int) => self.serialize_i64(casted_int),
                None => self.serialize_str(&int.to_string()),
            }
        }
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let fields = len.map(HashMap::with_capacity).unwrap_or_default();
        Ok(ValueMapSerializer {
            inner: self,
            fields,
            key: None,
        })
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        if name == super::timestamp::NEWTYPE_MARKER {
            value.serialize(newtype::MaybeTimestampSerializer::new(self))
        } else if name == Reference::NEWTYPE_MARKER {
            value.serialize(newtype::MaybeReferenceSerializer::new(self))
        } else {
            value.serialize(self)
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let values = len.map(Vec::with_capacity).unwrap_or_default();
        Ok(ValueSeqSerializer {
            inner: self,
            values,
        })
    }

    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::StringValue(s.to_owned())),
        }))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + std::fmt::Display,
    {
        thread_local! {
            static COLLECT_STR_BUF: RefCell<String> = RefCell::new(String::with_capacity(256));
        }

        COLLECT_STR_BUF.with_borrow_mut(|str_buf| {
            str_buf.clear();
            std::fmt::write(str_buf, format_args!("{value}")).map_err(ConvertError::ser)?;
            self.serialize_str(str_buf.as_str())
        })
    }
}

pub struct ValueSeqSerializer<N> {
    inner: ValueSerializer<N>,
    values: Vec<firestore::Value>,
}

impl<N> ser::SerializeSeq for ValueSeqSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match value.serialize(self.inner)? {
            Some(elem) => self.values.push(elem),
            None => N::handle_null(|elem| self.values.push(elem)),
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::ArrayValue(ArrayValue {
                values: self.values,
            })),
        }))
    }
}

impl<N> ser::SerializeTuple for ValueSeqSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<N> ser::SerializeTupleStruct for ValueSeqSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<N> ser::SerializeTupleVariant for ValueSeqSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

pub struct ValueMapSerializer<N> {
    inner: ValueSerializer<N>,
    fields: HashMap<String, firestore::Value>,
    key: Option<String>,
}

impl<N> ser::SerializeMap for ValueMapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_key<S>(&mut self, key: &S) -> Result<(), Self::Error>
    where
        S: Serialize + ?Sized,
    {
        key.serialize(KeyCapture(&mut self.key))
            .map_err(ConvertError::ser)
    }

    fn serialize_value<S>(&mut self, value: &S) -> Result<(), Self::Error>
    where
        S: Serialize + ?Sized,
    {
        let key = self
            .key
            .take()
            .ok_or_else(|| ConvertError::ser("map value has no key"))?;

        match value.serialize(self.inner)? {
            Some(value) => {
                self.fields.insert(key, value);
            }
            None => N::handle_null(|value| self.fields.insert(key, value)),
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Some(firestore::Value {
            value_type: Some(ValueType::MapValue(MapValue {
                fields: self.fields,
            })),
        }))
    }
}

impl<N> ser::SerializeStruct for ValueMapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match value.serialize(self.inner)? {
            Some(val) => {
                self.fields.insert(key.to_owned(), val);
            }
            None => N::handle_null(|val| self.fields.insert(key.to_owned(), val)),
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}

impl<N> ser::SerializeStructVariant for ValueMapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = Option<firestore::Value>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        <Self as ser::SerializeStruct>::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}
