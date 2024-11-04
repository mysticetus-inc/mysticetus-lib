use std::collections::HashMap;
use std::marker::PhantomData;

use protos::protobuf::value::Kind;
use protos::protobuf::{self, ListValue};
use serde::ser::{Error, Impossible};

use crate::error::{ConvertError, IntoError};
use crate::{IntoSpanner, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Serializer<Target>(PhantomData<fn(Target)>);

pub type ValueSerializer = Serializer<Value>;
pub type StringSerializer = Serializer<String>;

impl<Target> Default for Serializer<Target> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

macro_rules! impl_simple_error_fns {
    ($dst:ty => $($fn_name:ident($arg:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, arg: $arg) -> Result<Self::Ok, Self::Error> {
                Err(
                    IntoError::from_value(arg)
                        .reason(concat!("cannot serialize as a ", stringify!($dst)))
                        .into()
                )
            }
        )*
    };
}

macro_rules! impl_serialize_int_as_str {
    ($($fn_name:ident($arg:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, v: $arg) -> Result<Self::Ok, Self::Error> {
                Ok(itoa::Buffer::new().format(v).to_owned())
            }
        )*
    };
}

impl serde::Serializer for Serializer<String> {
    type Ok = String;
    type Error = ConvertError;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_owned())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_owned())
    }

    fn serialize_char(self, c: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0; 4];
        Ok(c.encode_utf8(&mut buf).to_owned())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(name.to_owned())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        match std::str::from_utf8(v) {
            Ok(s) => self.serialize_str(s),
            _ => Err(ConvertError::custom(
                "cannot convert non-utf8 bytes to a String",
            )),
        }
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    impl_serialize_int_as_str! {
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
    }

    impl_simple_error_fns! { String =>
        serialize_bool(bool),
        serialize_f32(f32),
        serialize_f64(f64),
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::custom("cannot serialize () as a String"))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ConvertError::custom(
            "cannot serialize a sequence as a String",
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ConvertError::custom("cannot serialize a map as a String"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::custom("cannot serialize None as a String"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ConvertError::custom("cannot serialize a tuple as a String"))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ConvertError::custom(format!(
            "cannot serialize a {name} as a String"
        )))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_struct(name, len)
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

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_struct(variant, len)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

macro_rules! impl_serialize_fn_via_into_spanner {
    ($($fn_name:ident($arg:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, v: $arg) -> Result<Self::Ok, Self::Error> {
                Ok(IntoSpanner::into_value(v))
            }
        )*
    };
}

impl<Target> Serializer<Target> {
    pub fn serialize<T: serde::Serialize>(value: T) -> Result<Target, ConvertError>
    where
        Self: serde::Serializer<Ok = Target, Error = ConvertError>,
    {
        value.serialize(Self(PhantomData))
    }
}

pub struct MapSerializer {
    fields: HashMap<String, protobuf::Value>,
    key: Option<String>,
}

pub struct SeqSerializer {
    values: Vec<protobuf::Value>,
}

impl serde::Serializer for Serializer<Value> {
    type Ok = Value;
    type Error = ConvertError;

    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;

    impl_serialize_fn_via_into_spanner! {
        serialize_bool(bool),
        serialize_char(char),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_str(&str),
        serialize_bytes(&[u8]),

    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::NULL)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::NULL)
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

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer {
            values: len.map(Vec::with_capacity).unwrap_or_default(),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SeqSerializer {
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            fields: len.map(HashMap::with_capacity).unwrap_or_default(),
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MapSerializer {
            fields: HashMap::with_capacity(len),
            key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(MapSerializer {
            fields: HashMap::with_capacity(len),
            key: None,
        })
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: std::fmt::Display,
    {
        Ok(Value::from(value.to_string()))
    }
}

impl serde::ser::SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = ConvertError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let val = value.serialize(ValueSerializer::default())?.into_protobuf();

        self.values.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value(Kind::ListValue(ListValue {
            values: self.values,
        })))
    }
}

impl serde::ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = ConvertError;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = ConvertError;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SeqSerializer {
    type Ok = Value;
    type Error = ConvertError;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = ConvertError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let key = key.serialize(StringSerializer::default())?;
        self.key = Some(key);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let key = self
            .key
            .take()
            .expect("called serialize_value before calling serialize_key");

        let value = value.serialize(ValueSerializer::default())?.into_protobuf();
        self.fields.insert(key, value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value(Kind::StructValue(protobuf::Struct {
            fields: self.fields,
        })))
    }
}

impl serde::ser::SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = ConvertError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let value = value.serialize(ValueSerializer::default())?.into_protobuf();
        self.fields.insert(key.to_owned(), value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}

impl serde::ser::SerializeStructVariant for MapSerializer {
    type Ok = Value;
    type Error = ConvertError;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeStruct::end(self)
    }
}
