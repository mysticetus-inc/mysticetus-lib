//! A serializer to create [`DocFields`] from [`Serialize`]-able types.

use std::marker::PhantomData;

use crate::error::SerError;

pub mod update;
pub mod write;

pub use update::Update;
pub use write::Write;

pub(super) type UpdateSerializer<W> = DocSerializer<Update<W>>;

pub(super) type WriteSerializer<W> = DocSerializer<Write<W>>;

// Outer serializer that ensure we're serializing a map, then defers to [`Kind`],
// which should be one of [`Update`] or [`Write`].
pub(super) struct DocSerializer<Kind> {
    marker: PhantomData<fn(Kind)>,
}

impl<Kind> DocSerializer<Kind> {
    pub(super) const NEW: Self = Self {
        marker: PhantomData,
    };
}

macro_rules! make_invalid_type_error {
    (format_args!($($other:tt)+)) => {
        Err(crate::error::SerError::Serialize(serde::ser::Error::custom(format!(
            "firestore documents should be a map, got {} instead",
            format_args!($($other)+)
        ))))
    };
    ($kind:ty) => {
        Err(crate::error::SerError::Serialize(serde::ser::Error::custom(concat!(
            "firestore documents should be a map, got '",
            stringify!($kind),
            "' instead",
        ))))
    };
    ($kind:literal) => {
        Err(crate::error::SerError::Serialize(serde::ser::Error::custom(concat!(
            "firestore documents should be a map, got '",
            $kind,
            "' instead",
        ))))
    };


}

macro_rules! impl_simple_invalid_serialize_fns {
    ($($name:ident($arg_ty:ty)),* $(,)?) => {
        $(
            fn $name(self, _: $arg_ty) -> Result<Self::Ok, Self::Error> {
                make_invalid_type_error!($arg_ty)
            }
        )*
    };
}

impl<Kind> serde::Serializer for DocSerializer<Kind>
where
    Kind: super::MapSerializerKind,
{
    type Ok = Kind::Output;
    type Error = SerError;

    type SerializeMap = Kind;
    type SerializeStruct = Kind;
    type SerializeStructVariant = Kind;

    type SerializeSeq = serde::ser::Impossible<Self::Ok, SerError>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, SerError>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, SerError>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, SerError>;

    impl_simple_invalid_serialize_fns! {
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8]),
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        make_invalid_type_error!("none")
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        make_invalid_type_error!("unit")
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        make_invalid_type_error!(format_args!("unit struct '{name}'"))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        make_invalid_type_error!(format_args!("unit variant '{name}::{variant}'"))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        make_invalid_type_error!("seq")
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        make_invalid_type_error!(format_args!("tuple (arity: {len})"))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        make_invalid_type_error!(format_args!("tuple struct '{name}'"))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        make_invalid_type_error!(format_args!(
            "tuple variant '{name}::{variant}' (arity: {len})"
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(Kind::new_with_len(len, ()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Kind::new_with_len(Some(len), ()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(Kind::new_with_len(Some(len), ()))
    }
}
