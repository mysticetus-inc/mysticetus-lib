use std::cell::{Cell, RefCell};
use std::fmt;

use serde::{de, ser, Serialize};

use crate::string_dst::StringDst;

pub struct SeededKeyCapture<S, D> {
    seed: S,
    string_dst: D,
}

impl<S, D> SeededKeyCapture<S, D> {
    #[inline]
    pub const fn new(seed: S, string_dst: D) -> Self {
        Self { seed, string_dst }
    }

    #[inline]
    pub const fn seed(&self) -> &S {
        &self.seed
    }

    #[inline]
    pub const fn seed_mut(&mut self) -> &mut S {
        &mut self.seed
    }

    #[inline]
    pub const fn string_dst(&self) -> &D {
        &self.string_dst
    }

    #[inline]
    pub const fn string_dst_mut(&mut self) -> &mut D {
        &mut self.string_dst
    }

    #[inline]
    pub fn into_inner(self) -> (S, D) {
        (self.seed, self.string_dst)
    }
}

impl<'de, S, Dst> de::DeserializeSeed<'de> for SeededKeyCapture<S, Dst>
where
    S: de::DeserializeSeed<'de>,
    Dst: StringDst,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.seed.deserialize(SeededKeyCapture {
            seed: deserializer,
            string_dst: self.string_dst,
        })
    }
}

macro_rules! impl_delegated_fn {
    ($($fn_name:ident),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>
            {
                self.seed.$fn_name(SeededKeyCapture::new(visitor, self.string_dst))
            }
        )*
    };
}

impl<'de, S, D> de::Deserializer<'de> for SeededKeyCapture<S, D>
where
    S: de::Deserializer<'de>,
    D: StringDst,
{
    type Error = S::Error;

    impl_delegated_fn! {
        deserialize_any,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_f32,
        deserialize_f64,
        deserialize_str,
        deserialize_bool,
        deserialize_char,
        deserialize_seq,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_map,
        deserialize_ignored_any,
        deserialize_identifier,
    }

    serde::serde_if_integer128! {
        impl_delegated_fn! {
            deserialize_i128,
            deserialize_u128,
        }
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.seed
            .deserialize_tuple(len, SeededKeyCapture::new(visitor, self.string_dst))
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.seed.deserialize_tuple_struct(
            name,
            len,
            SeededKeyCapture::new(visitor, self.string_dst),
        )
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.seed
            .deserialize_unit_struct(name, SeededKeyCapture::new(visitor, self.string_dst))
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.seed.deserialize_enum(
            name,
            variants,
            SeededKeyCapture::new(visitor, self.string_dst),
        )
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.seed.deserialize_struct(
            name,
            fields,
            SeededKeyCapture::new(visitor, self.string_dst),
        )
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.seed
            .deserialize_newtype_struct(name, SeededKeyCapture::new(visitor, self.string_dst))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.seed.is_human_readable()
    }
}

macro_rules! impl_visitor_delgate {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<E>(self, arg: $arg_ty) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                self.seed.$fn_name(arg)
            }
        )*
    };
}

impl<'de, S, D> de::Visitor<'de> for SeededKeyCapture<S, D>
where
    S: de::Visitor<'de>,
    D: StringDst,
{
    type Value = S::Value;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.seed.expecting(formatter)
    }

    #[inline]
    fn visit_borrowed_str<E>(mut self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.string_dst.handle_str(v);
        self.seed.visit_borrowed_str(v)
    }

    #[inline]
    fn visit_borrowed_bytes<E>(mut self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(s) = std::str::from_utf8(v) {
            self.string_dst.handle_str(s);
        }

        self.seed.visit_borrowed_bytes(v)
    }

    #[inline]
    fn visit_str<E>(mut self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = self.seed.visit_str(v)?;
        self.string_dst.handle_str(v);
        Ok(value)
    }

    #[inline]
    fn visit_string<E>(mut self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.string_dst.handle_str(&v);
        match self.seed.visit_string(v) {
            Ok(v) => Ok(v),
            Err(e) => {
                self.string_dst.clear();
                Err(e)
            }
        }
    }

    #[inline]
    fn visit_bytes<E>(mut self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(s) = std::str::from_utf8(v) {
            self.string_dst.handle_str(s);
        }

        self.seed.visit_bytes(v)
    }

    #[inline]
    fn visit_byte_buf<E>(mut self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(s) = std::str::from_utf8(&v) {
            self.string_dst.handle_str(s);
        }

        self.seed.visit_byte_buf(v)
    }

    #[inline]
    fn visit_char<E>(mut self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.string_dst.handle_char(v);
        self.seed.visit_char(v)
    }

    impl_visitor_delgate! {
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_f32(f32),
        visit_f64(f64),
        visit_bool(bool),
    }

    serde::serde_if_integer128! {
        impl_visitor_delgate! {
            visit_i128(i128),
            visit_u128(u128),
        }
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.seed.visit_none()
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.seed.visit_unit()
    }

    #[inline]
    fn visit_some<Des>(self, deserializer: Des) -> Result<Self::Value, Des::Error>
    where
        Des: serde::Deserializer<'de>,
    {
        self.seed
            .visit_some(SeededKeyCapture::new(deserializer, self.string_dst))
    }

    #[inline]
    fn visit_newtype_struct<Des>(self, deserializer: Des) -> Result<Self::Value, Des::Error>
    where
        Des: serde::Deserializer<'de>,
    {
        self.seed
            .visit_newtype_struct(SeededKeyCapture::new(deserializer, self.string_dst))
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.seed.visit_seq(seq)
    }

    #[inline]
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.seed.visit_map(map)
    }

    #[inline]
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.seed
            .visit_enum(SeededKeyCapture::new(data, self.string_dst))
    }
}

impl<'de, E, D> de::EnumAccess<'de> for SeededKeyCapture<E, D>
where
    E: de::EnumAccess<'de>,
    D: StringDst,
{
    type Variant = E::Variant;
    type Error = E::Error;

    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.seed
            .variant_seed(SeededKeyCapture::new(seed, self.string_dst))
    }
}

macro_rules! impl_delegated_ser_fns {
    ($($fn_name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name(self, arg: $arg_ty) -> Result<Self::Ok, Self::Error> {
                self.seed.$fn_name(arg)
            }
        )*
    };
}

impl<S, D> ser::Serializer for SeededKeyCapture<S, D>
where
    S: ser::Serializer,
    D: StringDst,
{
    type Ok = S::Ok;
    type Error = S::Error;

    type SerializeSeq = S::SerializeSeq;
    type SerializeMap = S::SerializeMap;
    type SerializeTuple = S::SerializeTuple;
    type SerializeStruct = S::SerializeStruct;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeStructVariant = S::SerializeStructVariant;

    impl_delegated_ser_fns! {
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
        impl_delegated_ser_fns! {
            serialize_i128(i128),
            serialize_u128(u128),
        }
    }

    #[inline]
    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.string_dst.handle_str(v);
        self.seed.serialize_str(v)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.seed.serialize_none()
    }

    #[inline]
    fn serialize_char(mut self, v: char) -> Result<Self::Ok, Self::Error> {
        self.string_dst.handle_char(v);
        self.seed.serialize_char(v)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.seed.serialize_unit()
    }

    #[inline]
    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if let Ok(s) = std::str::from_utf8(v) {
            self.string_dst.handle_str(s);
        }

        self.seed.serialize_bytes(v)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.seed.serialize_seq(len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.seed.serialize_map(len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.seed.serialize_tuple(len)
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.seed.serialize_unit_struct(name)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        T::serialize(value, self)
    }

    #[inline]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.seed.serialize_struct(name, len)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.seed.serialize_tuple_struct(name, len)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.seed
            .serialize_unit_variant(name, variant_index, variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.seed.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.seed
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.seed
            .serialize_struct_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.seed
            .serialize_newtype_variant(name, variant_index, variant, value)
    }
}

impl<T, D> Serialize for SeededKeyCapture<T, &Cell<D>>
where
    T: Serialize,
    D: StringDst + Default + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let wrapped = SeededKeyCapture::new(serializer, self.string_dst);
        self.seed.serialize(wrapped)
    }
}

impl<T, D> Serialize for SeededKeyCapture<T, &RefCell<D>>
where
    T: Serialize,
    D: StringDst,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let wrapped = SeededKeyCapture::new(serializer, self.string_dst);
        self.seed.serialize(wrapped)
    }
}
