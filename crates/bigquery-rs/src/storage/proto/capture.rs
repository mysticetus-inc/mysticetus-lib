//! A [`serde::Serializer`] that captures any serialized strings.

use serde::ser;

use super::EncodeError;

#[derive(Debug, PartialEq, Eq)]
pub struct Capture<'a>(pub &'a mut String);

macro_rules! impl_primitive_errs {
    ($($fn_name:ident($type:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, _: $type) -> Result<Self::Ok, Self::Error> {
                Err(EncodeError::InvalidType(stringify!($type)))
            }
        )*
    };
}

impl ser::Serializer for Capture<'_> {
    type Ok = ();
    type Error = EncodeError;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.push_str(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.0.push(v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        match std::str::from_utf8(v) {
            Ok(string) => self.serialize_str(string),
            Err(err) => Err(ser::Error::custom(err)),
        }
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        self.serialize_some(value)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        self.serialize_some(value)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.0.push_str(name);
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.0.push_str(variant);
        Ok(())
    }

    impl_primitive_errs! {
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
        serialize_bool(bool),
    }

    serde::serde_if_integer128! {
        impl_primitive_errs! {
            serialize_i128(i128),
            serialize_u128(u128),
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(EncodeError::InvalidType("None"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(EncodeError::InvalidType("()"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(EncodeError::InvalidType("sequence"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(EncodeError::InvalidType("map"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(EncodeError::InvalidType("tuple"))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(EncodeError::InvalidType(name))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(EncodeError::InvalidType(name))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(EncodeError::InvalidType(variant))
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
}
