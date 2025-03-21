use std::collections::HashMap;
use std::marker::PhantomData;

use protos::protobuf;
use protos::protobuf::value::Kind;
use serde::de::Error;

use crate::IntoSpanner;
use crate::error::ConvertError;
use crate::ty::SpannerType;
use crate::ty::markers::SpannerStruct;

pub struct EncodedStruct<T> {
    fields: HashMap<String, protobuf::Value>,
    _marker: PhantomData<T>,
}

impl<T: SpannerStruct> EncodedStruct<T> {
    pub fn from_fields(
        fields: impl IntoIterator<Item = (impl Into<String>, impl IntoSpanner)>,
    ) -> Self {
        Self {
            fields: fields
                .into_iter()
                .map(|(k, v)| (k.into(), v.into_value().into_protobuf()))
                .collect(),
            _marker: PhantomData,
        }
    }

    /*
    pub fn from_serializable(serializable: &T) -> Result<Self, ConvertError>
    where
        T: serde::Serialize,
    {
        serializable.serialize(StructSerializer::<T>(PhantomData))
    }
    */
}

impl<T: SpannerStruct> SpannerType for EncodedStruct<T> {
    type Nullable = typenum::False;
    type Type = crate::ty::markers::Struct<T>;
}

impl<T: SpannerStruct> IntoSpanner for EncodedStruct<T> {
    fn into_value(self) -> super::Value {
        super::Value(Kind::StructValue(protobuf::Struct {
            fields: self.fields,
        }))
    }
}

pub struct StructSerializer<T>(PhantomData<fn(T)>);

impl<T> StructSerializer<T> {
    #[inline]
    fn error(unexpected: serde::de::Unexpected<'_>) -> ConvertError {
        ConvertError::invalid_value(unexpected, &"a struct")
    }
}

impl<T: SpannerStruct> serde::Serializer for StructSerializer<T> {
    type Ok = EncodedStruct<T>;
    type Error = ConvertError;

    type SerializeStruct = SerializeStruct<T>;
    type SerializeStructVariant = SerializeStruct<T>;

    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct {
            fields: HashMap::with_capacity(len),
            _marker: PhantomData,
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_struct(name, len)
    }

    fn serialize_newtype_struct<V>(
        self,
        _name: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<V>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Signed(v as _)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Signed(v as _)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Signed(v as _)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Signed(v as _)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unsigned(v as _)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unsigned(v as _)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unsigned(v as _)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unsigned(v as _)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Float(v as _)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Float(v as _)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Char(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Str(v)))
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Bool(v)))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Bytes(v)))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Option))
    }

    fn serialize_some<V>(self, _value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        Err(Self::error(serde::de::Unexpected::Option))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unit))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unit))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Unit))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Seq))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Seq))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Seq))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Seq))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Self::error(serde::de::Unexpected::Map))
    }
}

pub struct SerializeStruct<T: SpannerStruct> {
    fields: HashMap<String, protobuf::Value>,
    _marker: PhantomData<fn(T)>,
}

impl<S: SpannerStruct> serde::ser::SerializeStruct for SerializeStruct<S> {
    type Ok = EncodedStruct<S>;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // if we cant find the field, skip it. If there's an error
        // due to type mismatches, we'll find it in 'end'
        let Some(field_def) = S::FIELDS.iter().find(|f| &f.name == key) else {
            return Ok(());
        };

        let value = crate::serde::ValueSerializer::serialize(value)?;

        if value.matches_type(&field_def.ty, true) == Some(false) {
            return Err(ConvertError::invalid_type(
                match &value.0 {
                    Kind::BoolValue(b) => serde::de::Unexpected::Bool(*b),
                    Kind::NumberValue(num) => serde::de::Unexpected::Float(*num),
                    Kind::StringValue(s) => serde::de::Unexpected::Str(s),
                    Kind::ListValue(_) => serde::de::Unexpected::Seq,
                    Kind::StructValue(_) => serde::de::Unexpected::Map,
                    Kind::NullValue(_) => serde::de::Unexpected::Option,
                },
                &format!("expected {key} to be {}", field_def.ty).as_str(),
            ));
        }

        if self
            .fields
            .insert(key.to_owned(), value.into_protobuf())
            .is_some()
        {
            return Err(ConvertError::duplicate_field(key));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<S: SpannerStruct> serde::ser::SerializeStructVariant for SerializeStruct<S> {
    type Ok = EncodedStruct<S>;
    type Error = ConvertError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        <Self as serde::ser::SerializeStruct>::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as serde::ser::SerializeStruct>::end(self)
    }
}
