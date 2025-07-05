use std::borrow::Cow;
use std::cell::RefCell;

use serde::ser::{self, Serialize, Serializer};
use serde_helpers::seeded_key_capture::SeededKeyCapture;

use crate::path::{ErrorPath, Track};

pub struct Delegate<'t, 'e, D, K, F> {
    pub(super) track: Cow<'t, Track<'t>>,
    pub(super) error_path: Cow<'e, ErrorPath>,
    pub(super) delegated: D,
    pub(super) key: K,
    pub(super) map_error: F,
}

pub trait MapError<In>: private::Sealed {
    type Out: ser::Error;

    fn map_error(&self, v: In, error_path: &ErrorPath) -> Self::Out;
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::BuildError {}
    impl Sealed for super::Identity {}
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct BuildError;

impl<E> MapError<E> for BuildError
where
    E: ser::Error,
{
    type Out = crate::Error<E>;

    fn map_error(&self, v: E, error_path: &ErrorPath) -> Self::Out {
        crate::Error::new(v, error_path.take())
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct Identity;

impl<E> MapError<E> for Identity
where
    E: ser::Error,
{
    type Out = E;

    fn map_error(&self, v: E, _: &ErrorPath) -> Self::Out {
        v
    }
}

macro_rules! wrap_delegated {
    ($self:expr, $delegated:expr) => {{
        Delegate {
            delegated: $delegated,
            track: Cow::Borrowed(&*$self.track),
            error_path: Cow::Borrowed(&*$self.error_path),
            key: $self.key,
            map_error: Identity,
        }
    }};
}

macro_rules! err_handler {
    ($self:expr) => {
        |err| {
            $self.error_path.set(&*$self.track);
            $self.map_error.map_error(err, &$self.error_path)
        }
    };
    ($self:expr,seq: $idx:expr) => {
        |err| {
            $self.error_path.set(&$self.track.add_seq_child($idx));
            $self.map_error.map_error(err, &$self.error_path)
        }
    };
    ($self:expr,map: $map:expr) => {
        |err| {
            let map = $map;
            let key = AsRef::<str>::as_ref(&**map);

            let child = if !key.is_empty() {
                $self.track.add_map_child(key)
            } else {
                $self.track.add_unknown_child()
            };

            $self.error_path.set(&child);
            $self.map_error.map_error(err, &$self.error_path)
        }
    };
    ($self:expr,unknown) => {
        |err| {
            $self.error_path.set(&$self.track.add_unknown_child());
            $self.map_error.map_error(err, &$self.error_path)
        }
    };
}

macro_rules! impl_delegated_ser_fns {
    ($($fn_name:ident($arg_type:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name(self, arg: $arg_type) -> Result<Self::Ok, Self::Error> {
                self.delegated.$fn_name(arg).map_err(err_handler!(self))
            }
        )*
    };
}

impl<'t, 'e, S, K, F> Serializer for Delegate<'t, 'e, S, K, F>
where
    S: Serializer,
    F: MapError<S::Error>,
{
    type Ok = S::Ok;
    type Error = F::Out;

    type SerializeSeq = Delegate<'t, 'e, S::SerializeSeq, usize, F>;
    type SerializeMap = Delegate<'t, 'e, S::SerializeMap, RefCell<String>, F>;
    type SerializeTuple = Delegate<'t, 'e, S::SerializeTuple, usize, F>;
    type SerializeStruct = Delegate<'t, 'e, S::SerializeStruct, &'static str, F>;
    type SerializeTupleStruct = Delegate<'t, 'e, S::SerializeTupleStruct, usize, F>;
    type SerializeTupleVariant = Delegate<'t, 'e, S::SerializeTupleVariant, usize, F>;
    type SerializeStructVariant = Delegate<'t, 'e, S::SerializeStructVariant, &'static str, F>;

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
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8]),
        serialize_unit_struct(&'static str),
    }

    serde::serde_if_integer128! {
        impl_delegated_ser_fns! {
            serialize_i128(i128),
            serialize_u128(u128),
        }
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.serialize_none().map_err(err_handler!(self))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.serialize_unit().map_err(err_handler!(self))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        T::serialize(value, self)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegated
            .serialize_newtype_struct(name, value)
            .map_err(err_handler!(self))
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegated
            .serialize_newtype_variant(name, variant_index, variant, value)
            .map_err(err_handler!(self))
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.delegated
            .serialize_unit_variant(name, variant_index, variant)
            .map_err(err_handler!(self))
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let delegated = self
            .delegated
            .serialize_seq(len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: 0,
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let delegated = self
            .delegated
            .serialize_map(len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: RefCell::new(String::new()),
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let delegated = self
            .delegated
            .serialize_tuple(len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: 0,
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }

    #[inline]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let delegated = self
            .delegated
            .serialize_struct(name, len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: "",
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let delegated = self
            .delegated
            .serialize_tuple_struct(name, len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: 0,
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let delegated = self
            .delegated
            .serialize_tuple_variant(name, variant_index, variant, len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: 0,
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let delegated = self
            .delegated
            .serialize_struct_variant(name, variant_index, variant, len)
            .map_err(err_handler!(self))?;

        Ok(Delegate {
            delegated,
            key: "",
            error_path: self.error_path,
            track: self.track,
            map_error: self.map_error,
        })
    }
}

impl<'t, 'e, S, F> ser::SerializeMap for Delegate<'t, 'e, S, RefCell<String>, F>
where
    S: ser::SerializeMap,
    F: MapError<S::Error>,
{
    type Ok = S::Ok;
    type Error = F::Out;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let delegated = Delegate {
            track: Cow::Borrowed(&*self.track),
            error_path: Cow::Borrowed(&*self.error_path),
            delegated: key,
            key: &self.key,
            map_error: Identity,
        };

        let wrapped = SeededKeyCapture::new(delegated, &self.key);

        self.delegated
            .serialize_key(&wrapped)
            .map_err(err_handler!(self, map: self.key.borrow()))
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let delegated = Delegate {
            track: Cow::Borrowed(&*self.track),
            error_path: Cow::Borrowed(&*self.error_path),
            delegated: value,
            key: &self.key,
            map_error: Identity,
        };

        self.delegated
            .serialize_value(&delegated)
            .map_err(err_handler!(self, map: self.key.borrow()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<'t, 'e, S, F> ser::SerializeSeq for Delegate<'t, 'e, S, usize, F>
where
    S: ser::SerializeSeq,
    F: MapError<S::Error>,
{
    type Ok = S::Ok;
    type Error = F::Out;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegated
            .serialize_element(&wrap_delegated!(self, value))
            .map_err(err_handler!(self, seq: self.key))?;

        self.key += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<D, F> ser::SerializeTuple for Delegate<'_, '_, D, usize, F>
where
    D: ser::SerializeTuple,
    F: MapError<D::Error>,
{
    type Ok = D::Ok;
    type Error = F::Out;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegated
            .serialize_element(&wrap_delegated!(self, value))
            .map_err(err_handler!(self, seq: self.key))?;

        self.key += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<'t, 'e, D, F> ser::SerializeStruct for Delegate<'t, 'e, D, &'static str, F>
where
    D: ser::SerializeStruct,
    F: MapError<D::Error>,
{
    type Ok = D::Ok;
    type Error = F::Out;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.key = key;

        self.delegated
            .serialize_field(key, &wrap_delegated!(self, value))
            .map_err(err_handler!(self, map: &key))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<'t, 'e, D, F> ser::SerializeTupleStruct for Delegate<'t, 'e, D, usize, F>
where
    D: ser::SerializeTupleStruct,
    F: MapError<D::Error>,
{
    type Ok = D::Ok;
    type Error = F::Out;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegated
            .serialize_field(&wrap_delegated!(self, value))
            .map_err(err_handler!(self, seq: self.key))?;

        self.key += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<'t, 'e, D, F> ser::SerializeTupleVariant for Delegate<'t, 'e, D, usize, F>
where
    D: ser::SerializeTupleVariant,
    F: MapError<D::Error>,
{
    type Ok = D::Ok;
    type Error = F::Out;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegated
            .serialize_field(&wrap_delegated!(self, value))
            .map_err(err_handler!(self, seq: self.key))?;

        self.key += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<'t, 'e, D, F> ser::SerializeStructVariant for Delegate<'t, 'e, D, &'static str, F>
where
    D: ser::SerializeStructVariant,
    F: MapError<D::Error>,
{
    type Ok = D::Ok;
    type Error = F::Out;

    fn serialize_field<T>(&mut self, field: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.key = field;

        self.delegated
            .serialize_field(field, &wrap_delegated!(self, value))
            .map_err(err_handler!(self, map: &field))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegated.end().map_err(err_handler!(self))
    }
}

impl<'t, 'e, T, K> Serialize for Delegate<'t, 'e, &T, K, Identity>
where
    T: Serialize + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapped = Delegate {
            key: &self.key,
            delegated: serializer,
            error_path: Cow::Borrowed(&*self.error_path),
            track: Cow::Borrowed(&*self.track),
            map_error: self.map_error,
        };

        T::serialize(self.delegated, wrapped).map_err(err_handler!(self))
    }
}
