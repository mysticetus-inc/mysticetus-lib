use std::marker::PhantomData;

use super::{QueryParameter, QueryParameterType, QueryParameterValue};
use crate::table::FieldType;

#[derive(Debug)]
pub struct QueryParameterSerializer<S>(PhantomData<fn(S)>);

impl<S> QueryParameterSerializer<S> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

#[inline]
fn new_scalar<S>(ty: FieldType, value: impl Into<serde_json::Value>) -> QueryParameter<S> {
    QueryParameter {
        name: None,
        parameter_type: QueryParameterType::Scalar(ty),
        parameter_value: QueryParameterValue::Scalar(value.into()),
    }
}

macro_rules! impl_serialize_scalar {
    ($($fn_name:ident($arg:ty, $field_ty:ident)),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name(self, v: $arg) -> Result<Self::Ok, Self::Error> {
                Ok(new_scalar(FieldType::$field_ty, v))
            }
        )*
    };
}

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("could not derive scalar type")]
    UnknownScalarType(QueryParameterValue),
    #[error("could not derive array element type")]
    UnknownArrayElementType(QueryParameterValue),
    #[error("could not derive map value type")]
    UnknownMapValueType(QueryParameterValue),
    #[error("found multiple incompatible nested types")]
    MixedNestedTypes,
}

impl serde::ser::Error for SerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Json(serde_json::Error::custom(msg))
    }
}

pub struct QueryParameterArraySerializer<S> {
    array: Vec<serde_json::Value>,
    element_ty: Option<QueryParameterType>,
    _marker: PhantomData<S>,
}

pub struct QueryParameterMapSerializer<S> {
    map: serde_json::Map<String, serde_json::Value>,
    next_key: Option<String>,
    value_ty: Option<QueryParameterType>,
    _marker: PhantomData<S>,
}

impl<S> serde::Serializer for QueryParameterSerializer<S> {
    type Ok = QueryParameter<S>;
    type Error = SerializeError;

    /*
    type SerializeMap = QueryParameterMapSerializer<S>;
    type SerializeStruct = QueryParameterMapSerializer<S>;
    type SerializeStructVariant = QueryParameterMapSerializer<S>;
    */

    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeSeq = QueryParameterArraySerializer<S>;
    type SerializeTuple = QueryParameterArraySerializer<S>;
    type SerializeTupleStruct = QueryParameterArraySerializer<S>;
    type SerializeTupleVariant = QueryParameterArraySerializer<S>;

    impl_serialize_scalar! {
        serialize_i8(i8, Integer),
        serialize_i16(i16, Integer),
        serialize_i32(i32, Integer),
        serialize_i64(i64, Integer),
        serialize_u8(u8, Integer),
        serialize_u16(u16, Integer),
        serialize_u32(u32, Integer),
        serialize_u64(u64, Integer),
        serialize_f32(f32, Float),
        serialize_f64(f64, Float),
        serialize_bool(bool, Bool),
        serialize_str(&str, String),
        serialize_bytes(&[u8], Bytes),
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(new_scalar(FieldType::String, name))
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0; 4];
        let s = v.encode_utf8(&mut buf).to_owned();
        Ok(new_scalar(FieldType::String, s))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeError::UnknownScalarType(
            QueryParameterValue::Scalar(serde_json::Value::Null),
        ))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(new_scalar(FieldType::String, variant))
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(QueryParameterArraySerializer {
            array: len.map(Vec::with_capacity).unwrap_or_default(),
            element_ty: None,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        /*
        Ok(QueryParameterMapSerializer {
            map: len.map(serde_json::Map::with_capacity).unwrap_or_default(),
            value_ty: None,
            _marker: PhantomData,
        })
        */
        todo!("get SerializeMap/Struct serializers working")
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + std::fmt::Display,
    {
        Ok(new_scalar(FieldType::String, value.to_string()))
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

fn param_value_into_value(param: QueryParameterValue) -> serde_json::Value {
    match param {
        QueryParameterValue::Scalar(value) => value,
        QueryParameterValue::Array(arr) => serde_json::Value::Array(arr),
        QueryParameterValue::Struct(map) => serde_json::Value::Object(map),
        QueryParameterValue::Range(_) => todo!("ranges arent supported for serialization yet"),
    }
}

impl<S> serde::ser::SerializeSeq for QueryParameterArraySerializer<S> {
    type Ok = QueryParameter<S>;
    type Error = SerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        match value.serialize(QueryParameterSerializer::<S>(PhantomData)) {
            Ok(QueryParameter {
                parameter_type,
                parameter_value,
                ..
            }) => {
                self.array.push(param_value_into_value(parameter_value));
                match self.element_ty {
                    Some(ref existing) if existing == &parameter_type => (),
                    Some(_) => return Err(SerializeError::MixedNestedTypes),
                    None => self.element_ty = Some(parameter_type),
                }
                Ok(())
            }
            // if we get one of these, we could reasonably get the information from an element we
            // havent been given yet (i.e an array of nullable integers, starting with
            // null)
            Err(SerializeError::UnknownArrayElementType(param))
            | Err(SerializeError::UnknownScalarType(param))
            | Err(SerializeError::UnknownMapValueType(param)) => {
                self.array.push(param_value_into_value(param));
                Ok(())
            }
            Err(other_err) => Err(other_err),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let value = QueryParameterValue::Array(self.array);

        match self.element_ty {
            None => Err(SerializeError::UnknownArrayElementType(value)),
            Some(element_ty) => Ok(QueryParameter {
                name: None,
                parameter_type: QueryParameterType::Array {
                    element_type: Box::new(element_ty),
                },
                parameter_value: value,
            }),
        }
    }
}

impl<S> serde::ser::SerializeTuple for QueryParameterArraySerializer<S> {
    type Ok = QueryParameter<S>;
    type Error = SerializeError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<S> serde::ser::SerializeTupleStruct for QueryParameterArraySerializer<S> {
    type Ok = QueryParameter<S>;
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<S> serde::ser::SerializeTupleVariant for QueryParameterArraySerializer<S> {
    type Ok = QueryParameter<S>;
    type Error = SerializeError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<S> serde::ser::SerializeMap for QueryParameterMapSerializer<S> {
    type Ok = QueryParameter<S>;
    type Error = SerializeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let key = self
            .next_key
            .take()
            .expect("serialize_key not called first");

        match value.serialize(QueryParameterSerializer::<S>(PhantomData)) {
            Ok(QueryParameter {
                parameter_type,
                parameter_value,
                ..
            }) => {
                self.map
                    .insert(key, param_value_into_value(parameter_value));

                match self.value_ty {
                    Some(ref existing) if existing == &parameter_type => (),
                    Some(_) => return Err(SerializeError::MixedNestedTypes),
                    None => self.value_ty = Some(parameter_type),
                }
                Ok(())
            }
            // if we get one of these, we could reasonably get the information from an element we
            // havent been given yet (i.e an array of nullable integers, starting with
            // null)
            Err(SerializeError::UnknownArrayElementType(param))
            | Err(SerializeError::UnknownScalarType(param))
            | Err(SerializeError::UnknownMapValueType(param)) => {
                self.map.insert(key, param_value_into_value(param));
                Ok(())
            }
            Err(other_err) => Err(other_err),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let value = QueryParameterValue::Struct(self.map);
        todo!()
    }
}
