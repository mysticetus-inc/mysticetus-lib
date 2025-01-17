use std::borrow::Cow;
use std::cell::RefCell;

use serde::{Serialize, ser};

use crate::path::{ErrorPath, Track};

mod delegate;
use delegate::{BuildError, Delegate, Identity, MapError};

/// A serializer that wraps another. The wrapping layer modifies the errors to track paths.
pub struct Serializer<'t, 'e, S> {
    serializer: S,
    error_path: Cow<'e, ErrorPath>,
    track: Track<'t>,
}

impl<'t, 'e, S> Serializer<'t, 'e, S> {
    /// Wraps a given serializer, modifying the errors to track paths.
    pub fn new(serializer: S) -> Self {
        Self {
            track: Track::Root,
            serializer,
            error_path: Cow::Owned(ErrorPath::new(None)),
        }
    }
}

macro_rules! build_err {
    ($self:expr) => {{ |err| BuildError.map_error(err, &$self.error_path) }};
}

macro_rules! build_delegate {
    ($self:expr, $delegated:expr,key: $key:expr) => {{
        Delegate {
            delegated: $delegated,
            key: $key,
            map_error: BuildError,
            track: Cow::Owned($self.track),
            error_path: $self.error_path,
        }
    }};
    ($self:expr, $delegated:expr,map_error: $map_err:expr) => {{
        Delegate {
            delegated: $delegated,
            key: (),
            map_error: $map_err,
            track: Cow::Owned($self.track),
            error_path: $self.error_path,
        }
    }};
}

macro_rules! impl_simple_serializer_fns {
    ($($fn_name:ident ($($arg:ident: $arg_ty:ty)?)),* $(,)?) => {
        $(
            fn $fn_name(self $(, $arg: $arg_ty)?) -> Result<Self::Ok, Self::Error> {
                self.serializer.$fn_name($($arg)?)
                    .map_err(build_err!(self))
            }
        )*
    };
}

impl<'t, 'e, S> ser::Serializer for Serializer<'t, 'e, S>
where
    S: ser::Serializer,
{
    type Ok = S::Ok;
    type Error = crate::Error<S::Error>;

    type SerializeSeq = Delegate<'t, 'e, S::SerializeSeq, usize, BuildError>;
    type SerializeMap = Delegate<'t, 'e, S::SerializeMap, RefCell<String>, BuildError>;
    type SerializeTuple = Delegate<'t, 'e, S::SerializeTuple, usize, BuildError>;
    type SerializeStruct = Delegate<'t, 'e, S::SerializeStruct, &'static str, BuildError>;
    type SerializeTupleStruct = Delegate<'t, 'e, S::SerializeTupleStruct, usize, BuildError>;
    type SerializeTupleVariant = Delegate<'t, 'e, S::SerializeTupleVariant, usize, BuildError>;
    type SerializeStructVariant =
        Delegate<'t, 'e, S::SerializeStructVariant, &'static str, BuildError>;

    impl_simple_serializer_fns! {
        serialize_i8(v: i8),
        serialize_i16(v: i16),
        serialize_i32(v: i32),
        serialize_i64(v: i64),
        serialize_u8(v: u8),
        serialize_u16(v: u16),
        serialize_u32(v: u32),
        serialize_u64(v: u64),
        serialize_f32(v: f32),
        serialize_f64(v: f64),
        serialize_bool(v: bool),
        serialize_char(v: char),
        serialize_str(v: &str),
        serialize_bytes(v: &[u8]),
        serialize_unit(),
        serialize_none(),
    }

    serde::serde_if_integer128! {
        impl_simple_serializer_fns! {
            serialize_i128(v: i128),
            serialize_u128(v: u128),
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let delegated = self
            .serializer
            .serialize_seq(len)
            .map_err(build_err!(self))?;
        Ok(build_delegate!(self, delegated, key: 0))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let delegated = self
            .serializer
            .serialize_map(len)
            .map_err(build_err!(self))?;
        Ok(build_delegate!(
            self,
            delegated,
            key: RefCell::new(String::new())
        ))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let delegated = build_delegate!(self, value, map_error: Identity);
        self.serializer
            .serialize_some(&delegated)
            .map_err(build_err!(delegated))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let delegated = self
            .serializer
            .serialize_tuple(len)
            .map_err(build_err!(self))?;
        Ok(build_delegate!(self, delegated, key: 0))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serializer
            .serialize_unit_struct(name)
            .map_err(build_err!(self))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let delegated = self
            .serializer
            .serialize_struct(name, len)
            .map_err(build_err!(self))?;

        Ok(build_delegate!(self, delegated, key: ""))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let delegated = self
            .serializer
            .serialize_tuple_struct(name, len)
            .map_err(build_err!(self))?;

        Ok(build_delegate!(self, delegated, key: 0))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let delegated = build_delegate!(self, value, map_error: Identity);

        self.serializer
            .serialize_newtype_struct(name, &delegated)
            .map_err(build_err!(delegated))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serializer
            .serialize_unit_variant(name, variant_index, variant)
            .map_err(build_err!(self))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let delegated = self
            .serializer
            .serialize_tuple_variant(name, variant_index, variant, len)
            .map_err(build_err!(self))?;

        Ok(build_delegate!(self, delegated, key: 0))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let delegated = self
            .serializer
            .serialize_struct_variant(name, variant_index, variant, len)
            .map_err(build_err!(self))?;

        Ok(build_delegate!(self, delegated, key: ""))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let wrapped = build_delegate!(self, value, map_error: Identity);
        self.serializer
            .serialize_newtype_variant(name, variant_index, variant, &wrapped)
            .map_err(build_err!(wrapped))
    }
}
