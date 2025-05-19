use std::mem::transmute;

use bytes::{BufMut, BytesMut};
use protos::bigquery_storage::table_field_schema::Mode;
use serde::{Serialize, Serializer, ser};

use super::EncodeError;
use super::encode::{Varint, WireType};
use crate::write::{FieldInfo, Schema};

#[derive(Debug)]
pub struct ProtoSerializer<'a, B: ?Sized = BytesMut> {
    row: &'a mut B,
    schema: &'a Schema,
}

impl<'a, B: BufMut + ?Sized> ProtoSerializer<'a, B> {
    pub fn new(row: &'a mut B, schema: &'a Schema) -> Self {
        Self { row, schema }
    }

    pub fn serialize_row<T>(&mut self, row: &T) -> Result<(), EncodeError>
    where
        T: Serialize + ?Sized,
    {
        row.serialize(&mut *self)
    }
}

impl ser::Error for EncodeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Misc(msg.to_string().into_boxed_str())
    }
}

macro_rules! impl_primitive_ser_fns {
    ($($fn_name:ident($type:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, _: $type) -> Result<Self::Ok, Self::Error> {
                Err(EncodeError::InvalidType(stringify!($type)))
            }
        )*
    };
}

impl<'a, 'b, B: ?Sized + BufMut> Serializer for &'b mut ProtoSerializer<'a, B>
where
    'a: 'b,
{
    type Ok = ();
    type Error = EncodeError;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ProtoMapSerializer<'a, 'b, B>;
    type SerializeStruct = Self;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Self;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;

    impl_primitive_ser_fns! {
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_bool(bool),
        serialize_bytes(&[u8]),
        serialize_str(&str),
    }

    serde::serde_if_integer128! {
        impl_primitive_ser_fns! {
            serialize_i128(i128),
            serialize_u128(u128),
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(EncodeError::InvalidType("None"))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(EncodeError::InvalidType("()"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(EncodeError::InvalidType("seq"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ProtoMapSerializer {
            parent: self,
            key_buf: String::new(),
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(EncodeError::InvalidType("tuple"))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(EncodeError::InvalidType(name))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(EncodeError::InvalidType("tuple"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(EncodeError::InvalidType(variant))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(EncodeError::InvalidType("tuple"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
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
}

impl ser::SerializeSeq for &mut ProtoSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.serialize_row(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct ProtoMapSerializer<'a, 'b, B: ?Sized> {
    parent: &'b mut ProtoSerializer<'a, B: ?Sized>,
    key_buf: String,
}

impl ser::SerializeMap for ProtoMapSerializer<'_, '_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.key_buf.clear();
        key.serialize(super::capture::Capture(&mut self.key_buf))?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        if self.key_buf.is_empty() {
            return Err(EncodeError::Misc("invalid/missing key".into()));
        }

        let mut serializer = ProtoValueSerializer::new(&self.key_buf, self.parent)?;

        value.serialize(&mut serializer)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut ProtoSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.serialize_row(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut ProtoSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.serialize_row(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut ProtoSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let mut serializer = ProtoValueSerializer::new(&key, self)?;

        value.serialize(&mut serializer)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut ProtoSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeStruct::end(self)
    }
}

impl ser::SerializeTupleStruct for &mut ProtoSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_row(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProtoValueSerializer<'a, B: ?Sized = BytesMut> {
    buf: &'a mut B,
    field: &'a FieldInfo,
}

impl<'a, B: ?Sized> ProtoValueSerializer<'a, B> {
    fn new<S>(field: &'a S, parent: &'a mut ProtoSerializer<'_, B>) -> Result<Self, EncodeError>
    where
        S: AsRef<str>,
    {
        let field_name = field.as_ref();
        let field = parent
            .schema
            .get_field(field_name)
            .ok_or_else(|| EncodeError::MissingField(field_name.into()))?;

        Ok(Self {
            field,
            buf: parent.row,
        })
    }

    fn serialize_length_delimited<S>(&mut self, src: S)
    where
        S: AsRef<[u8]>,
    {
        let src = src.as_ref();

        if !src.is_empty() || matches!(self.field.mode(), Mode::Required) {
            self.field.encode_tag(&mut self.buf);
            Varint::from_unsigned(src.len()).encode(&mut self.buf);
            self.buf.extend_from_slice(src);
        }
    }

    fn serialize_varint(&mut self, varint: Varint) {
        self.field.encode_tag(&mut self.buf);
        varint.encode(&mut self.buf);
    }

    fn serialize_bits64(&mut self, bits: u64) {
        self.field.encode_tag(&mut self.buf);
        self.buf.put_u64_le(bits);
    }

    fn serialize_bits32(&mut self, bits: u32) {
        self.field.encode_tag(&mut self.buf);
        self.buf.put_u32_le(bits);
    }

    fn serialize_signed(&mut self, signed: isize) {
        match self.field.field().wire_type() {
            WireType::Varint => self.serialize_varint(Varint::from_signed(signed)),
            // SAFETY: simple bitcasting
            WireType::Bits64 => self.serialize_bits64(unsafe { transmute(signed as i64) }),
            WireType::LengthDelimited => self.serialize_length_delimited(signed.to_string()),
            WireType::StartGroup | WireType::EndGroup => {
                unimplemented!("start/end groups not supported")
            }
            // SAFETY: simple bitcasting
            WireType::Bits32 => self.buf.put_u32(unsafe { transmute(signed as i32) }),
        }
    }

    fn serialize_unsigned(&mut self, unsigned: usize) {
        match self.field.field().wire_type() {
            WireType::Varint => self.serialize_varint(Varint::from_unsigned(unsigned)),
            WireType::Bits64 => self.serialize_bits64(unsigned as u64),
            WireType::LengthDelimited => self.serialize_length_delimited(unsigned.to_string()),
            WireType::StartGroup | WireType::EndGroup => {
                unimplemented!("start/end groups not supported")
            }
            WireType::Bits32 => self.serialize_bits32(unsigned as u32),
        }
    }
}

macro_rules! impl_int_fns {
    (signed: $($fn_name:ident($type:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, v: $type) -> Result<Self::Ok, Self::Error> {
                self.serialize_signed(v as isize);
                Ok(())
            }
        )*
    };
    (unsigned: $($fn_name:ident($type:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, v: $type) -> Result<Self::Ok, Self::Error> {
                self.serialize_unsigned(v as usize);
                Ok(())
            }
        )*
    };
}

impl<'a> Serializer for &mut ProtoValueSerializer<'a> {
    type Ok = ();
    type Error = EncodeError;

    type SerializeSeq = Self;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Self;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_length_delimited(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serialize_length_delimited(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        match self.field.field().wire_type() {
            WireType::Varint => self.serialize_varint(Varint::from_signed(v as isize)),
            WireType::Bits64 => self.serialize_bits64((v as f64).to_bits()),
            WireType::LengthDelimited => self.serialize_length_delimited(v.to_string()),
            WireType::StartGroup | WireType::EndGroup => {
                unimplemented!("start/end groups not supported")
            }
            WireType::Bits32 => self.serialize_bits32(v.to_bits()),
        }

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        match self.field.field().wire_type() {
            WireType::Varint => self.serialize_varint(Varint::from_signed(v as isize)),
            WireType::Bits64 => self.serialize_bits64(v.to_bits()),
            WireType::LengthDelimited => self.serialize_length_delimited(v.to_string()),
            WireType::StartGroup | WireType::EndGroup => {
                unimplemented!("start/end groups not supported")
            }
            WireType::Bits32 => self.serialize_bits32((v as f32).to_bits()),
        }

        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        match self.field.field().wire_type() {
            WireType::Varint => self.serialize_varint(Varint::from_unsigned(v as usize)),
            WireType::Bits64 => self.serialize_bits64(v as u64),
            WireType::LengthDelimited => {
                // 4 bytes is enough to encode any char
                let mut buf = [0; 4];
                self.serialize_length_delimited(v.encode_utf8(&mut buf));
            }
            WireType::StartGroup | WireType::EndGroup => {
                unimplemented!("start/end groups not supported")
            }
            WireType::Bits32 => self.serialize_bits32(v as u32),
        }

        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        match self.field.field().wire_type() {
            WireType::Varint => self.serialize_varint(Varint::from_bool(v)),
            WireType::Bits64 => self.serialize_bits64(v as u64),
            WireType::LengthDelimited => {
                self.serialize_length_delimited(if v { "true" } else { "false" })
            }
            WireType::StartGroup | WireType::EndGroup => {
                unimplemented!("start/end groups not supported")
            }
            WireType::Bits32 => self.serialize_bits32(v as u32),
        }
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if matches!(self.field.mode(), Mode::Required) {
            return Err(EncodeError::MissingField(self.field.name().into()));
        }

        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
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

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_length_delimited(variant);
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_length_delimited(name);
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(EncodeError::InvalidType("map"))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(EncodeError::InvalidType(name))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(EncodeError::InvalidType(variant))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    impl_int_fns! {
        signed:
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
    }

    impl_int_fns! {
        unsigned:
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
    }

    serde::serde_if_integer128! {
        impl_int_fns! { signed: serialize_i128(i128) }
        impl_int_fns! { unsigned: serialize_u128(u128) }
    }
}

impl ser::SerializeSeq for &mut ProtoValueSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut ProtoValueSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        // defer to existing impl
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // defer to existing impl
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for &mut ProtoValueSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        // defer to existing impl
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // defer to existing impl
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for &mut ProtoValueSerializer<'_> {
    type Ok = ();
    type Error = EncodeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        // defer to existing impl
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // defer to existing impl
        ser::SerializeSeq::end(self)
    }
}
