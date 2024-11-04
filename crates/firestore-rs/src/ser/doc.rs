//! A serializer to create [`DocFields`] from [`Serialize`]-able types.
use std::marker::PhantomData;

use protos::firestore::DocumentMask;
use serde::ser::{self, Impossible};
use serde::{Serialize, Serializer};

use super::{DocFields, MapSerializer, NullStrategy};
use crate::ConvertError;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DocSerializer<N>(PhantomData<N>);

macro_rules! impl_err_ser_fns {
    ($(($fn_name:ident, $arg_type:ty)),* $(,)?) => {
        $(
            fn $fn_name(self, _: $arg_type) -> Result<Self::Ok, Self::Error> {
                Err(ConvertError::ser("cannot serialize a single value, must be a struct/map"))
            }
        )*
    };
}

impl<N> Serializer for DocSerializer<N>
where
    N: NullStrategy,
{
    type Ok = DocFields;

    type Error = ConvertError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    type SerializeMap = DocMapSerializer<N>;
    type SerializeStruct = DocMapSerializer<N>;
    type SerializeStructVariant = DocMapSerializer<N>;

    impl_err_ser_fns! {
        (serialize_bool, bool),
        (serialize_i8, i8),
        (serialize_i16, i16),
        (serialize_i32, i32),
        (serialize_i64, i64),
        (serialize_i128, i128),
        (serialize_u8, u8),
        (serialize_u16, u16),
        (serialize_u32, u32),
        (serialize_u64, u64),
        (serialize_u128, u128),
        (serialize_f32, f32),
        (serialize_f64, f64),
        (serialize_char, char),
        (serialize_str, &str),
        (serialize_bytes, &[u8]),
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::ser(
            "cannot serialize a single value, must be a struct/map",
        ))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::ser(
            "cannot serialize a single value, must be a struct/map",
        ))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::ser(
            "cannot serialize a single value, must be a struct/map",
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _var_idx: u32,
        _var_name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ConvertError::ser(
            "cannot serialize a single value, must be a struct/map",
        ))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
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
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ConvertError::ser(
            "document cannot be serialized to a sequence",
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ConvertError::ser(
            "document cannot be serialized to a sequence",
        ))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ConvertError::ser(
            "document cannot be serialized to a sequence",
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _var_idx: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ConvertError::ser(
            "document cannot be serialized to a sequence",
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(DocMapSerializer {
            _marker: PhantomData,
            map_ser: MapSerializer::new(len),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(DocMapSerializer {
            _marker: PhantomData,
            map_ser: MapSerializer::new(Some(len)),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(DocMapSerializer {
            _marker: PhantomData,
            map_ser: MapSerializer::new(Some(len)),
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocMapSerializer<N> {
    map_ser: MapSerializer<N>,
    _marker: std::marker::PhantomData<N>,
}

impl<N> ser::SerializeMap for DocMapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = DocFields;
    type Error = ConvertError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.map_ser.serialize_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.map_ser.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let fields = self.map_ser.end()?;

        let field_paths = super::build_mask(&fields);

        Ok(DocFields {
            field_mask: DocumentMask { field_paths },
            fields,
        })
    }
}

impl<N> ser::SerializeStruct for DocMapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = DocFields;
    type Error = ConvertError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.map_ser.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let fields = self.map_ser.end()?;
        let field_paths = super::build_mask(&fields);

        Ok(DocFields {
            field_mask: DocumentMask { field_paths },
            fields,
        })
    }
}

impl<N> ser::SerializeStructVariant for DocMapSerializer<N>
where
    N: NullStrategy,
{
    type Ok = DocFields;
    type Error = ConvertError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.map_ser.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let fields = self.map_ser.end()?;
        let field_paths = super::build_mask(&fields);

        Ok(DocFields {
            field_mask: DocumentMask { field_paths },
            fields,
        })
    }
}
