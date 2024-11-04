//! Manual serializer that partially replicates the `#[serde(flatten)]` field attribute.
//!
//! Implementation is a slightly modified copy of the [internal serde impl]
//! that powers `#[serde(flatten)]`.
//!  
//! [internal serde impl]: https://github.com/serde-rs/serde/blob/f6623a36548cfce02f880a33c6d2f420934c95c5/serde/src/private/ser.rs#L991-L1337

use serde::ser::{self, Serialize, SerializeMap, Serializer};

pub struct FlatMapSerializer<'a, M: 'a>(pub &'a mut M);

impl<'a, M> FlatMapSerializer<'a, M>
where
    M: SerializeMap + 'a,
{
    fn bad_type(what: &'static str) -> M::Error {
        ser::Error::custom(format_args!(
            "can only flatten structs and maps (got {})",
            what
        ))
    }
}

impl<'a, M> Serializer for FlatMapSerializer<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    type SerializeSeq = ser::Impossible<Self::Ok, M::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, M::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, M::Error>;
    type SerializeMap = FlatMapSerializeMap<'a, M>;
    type SerializeStruct = FlatMapSerializeStruct<'a, M>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, M::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, M::Error>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a boolean"))
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an integer"))
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a float"))
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a float"))
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a char"))
    }

    fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a string"))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a byte array"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("a unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type("an enum"))
    }

    fn serialize_newtype_struct<T>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_key(variant)?;
        self.0.serialize_value(value)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Self::bad_type("a sequence"))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Self::bad_type("a tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Self::bad_type("a tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Self::bad_type("an enum tuple variant"))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(FlatMapSerializeMap(self.0))
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(FlatMapSerializeStruct(self.0))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _inner_variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Self::bad_type("an enum struct variant"))
    }
}

pub struct FlatMapSerializeMap<'a, M: 'a>(&'a mut M);

impl<'a, M> ser::SerializeMap for FlatMapSerializeMap<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_value(value)
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.0.serialize_entry(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct FlatMapSerializeStruct<'a, M: 'a>(&'a mut M);

impl<'a, M> ser::SerializeStruct for FlatMapSerializeStruct<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_entry(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
