use std::collections::HashMap;
use std::marker::PhantomData;

use protos::firestore;
use protos::firestore::document_transform::FieldTransform;
use protos::firestore::document_transform::field_transform::TransformType;
use protos::firestore::value::ValueType::NullValue;

use crate::DocFields;
use crate::error::SerError;
use crate::ser::field_transform::FieldTransforms;
use crate::ser::value::{SerializedValueKind, ValueSerializer};
use crate::ser::{MapSerializerKind, WriteKind};

pub struct Write<W: WriteKind> {
    transforms: FieldTransforms,
    fields: HashMap<String, firestore::Value>,
    _marker: PhantomData<fn(W)>,
}

impl<W: WriteKind> MapSerializerKind for Write<W> {
    type Output = (DocFields, Vec<FieldTransform>);

    fn new_with_len(len: Option<usize>, _: ()) -> Self {
        Self {
            transforms: FieldTransforms::default(),
            fields: HashMap::with_capacity(len.unwrap_or(8)),
            _marker: PhantomData,
        }
    }
}

impl<W: WriteKind> serde::ser::SerializeMap for Write<W> {
    type Ok = (DocFields, Vec<FieldTransform>);
    type Error = SerError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut self.transforms.path)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let mut serializer = ValueSerializer::<W, TransformType>::new(&mut self.transforms);

        match value.serialize(&mut serializer)? {
            SerializedValueKind::Value(value) => {
                assert_eq!(self.transforms.path.len(), 1);

                if W::MERGE && matches!(value, NullValue(_)) {
                    self.transforms.path.pop();
                    return Ok(());
                }

                let field = self.transforms.path.pop_take().unwrap();

                self.fields.insert(
                    field.into_owned(),
                    firestore::Value {
                        value_type: Some(value),
                    },
                );
            }
            SerializedValueKind::Transform(transform) => {
                self.transforms.transforms.push(FieldTransform {
                    field_path: self.transforms.path.make_path(),
                    transform_type: Some(transform),
                });
            }
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((
            DocFields {
                fields: self.fields,
                field_mask: None,
            },
            self.transforms.transforms,
        ))
    }
}

impl<W: WriteKind> serde::ser::SerializeStruct for Write<W> {
    type Ok = (DocFields, Vec<FieldTransform>);
    type Error = SerError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        assert_eq!(self.transforms.path.len(), 0);

        self.transforms.path.push_static(key);

        serde::ser::SerializeMap::serialize_value(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}

impl<W: WriteKind> serde::ser::SerializeStructVariant for Write<W> {
    type Ok = (DocFields, Vec<FieldTransform>);
    type Error = SerError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}
