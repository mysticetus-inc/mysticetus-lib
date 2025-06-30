use protos::firestore;
use protos::firestore::document_transform::FieldTransform;
use protos::firestore::document_transform::field_transform::{ServerValue, TransformType};

use crate::error::{InvalidTransform, SerError};
use crate::ser::WriteKind;

#[derive(Debug, Default)]
pub(crate) struct NoFieldTransforms;

impl Into<Vec<FieldTransform>> for NoFieldTransforms {
    fn into(self) -> Vec<FieldTransform> {
        Vec::new()
    }
}

impl From<&mut NoFieldTransforms> for NoFieldTransforms {
    fn from(_: &mut NoFieldTransforms) -> Self {
        NoFieldTransforms
    }
}

#[derive(Debug, Default)]
pub(crate) struct FieldTransforms {
    pub(super) path: super::path::Path,
    pub(super) transforms: Vec<FieldTransform>,
}

impl Into<Vec<FieldTransform>> for FieldTransforms {
    fn into(self) -> Vec<FieldTransform> {
        self.transforms
    }
}

pub(crate) trait Transform: Sized {
    type Container<'a>;

    fn reborrow<'a, 'new>(container: &'new mut Self::Container<'a>) -> Self::Container<'new>;

    fn server_timestamp() -> Result<Self, SerError>;

    fn increment<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, SerError>;
    fn minimum<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, SerError>;
    fn maximum<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, SerError>;

    fn append_missing_elements<T: serde::Serialize + ?Sized, W: WriteKind>(
        value: &T,
    ) -> Result<Self, SerError>;

    fn remove_all_from_array<T: serde::Serialize + ?Sized, W: WriteKind>(
        value: &T,
    ) -> Result<Self, SerError>;
}

impl Transform for std::convert::Infallible {
    type Container<'a> = NoFieldTransforms;

    #[inline]
    fn reborrow<'a, 'new>(_: &'new mut Self::Container<'a>) -> Self::Container<'new> {
        NoFieldTransforms
    }

    #[inline]
    fn server_timestamp() -> Result<Self, SerError> {
        Err(SerError::FieldTransformsNotSupported)
    }

    #[inline]
    fn increment<T: serde::Serialize + ?Sized>(_: &T) -> Result<Self, SerError> {
        Err(SerError::FieldTransformsNotSupported)
    }

    #[inline]
    fn minimum<T: serde::Serialize + ?Sized>(_: &T) -> Result<Self, SerError> {
        Err(SerError::FieldTransformsNotSupported)
    }

    #[inline]
    fn maximum<T: serde::Serialize + ?Sized>(_: &T) -> Result<Self, SerError> {
        Err(SerError::FieldTransformsNotSupported)
    }

    #[inline]
    fn remove_all_from_array<T: serde::Serialize + ?Sized, W: WriteKind>(
        _: &T,
    ) -> Result<Self, SerError> {
        Err(SerError::FieldTransformsNotSupported)
    }

    #[inline]
    fn append_missing_elements<T: serde::Serialize + ?Sized, W: WriteKind>(
        _: &T,
    ) -> Result<Self, SerError> {
        Err(SerError::FieldTransformsNotSupported)
    }
}

impl Transform for TransformType {
    type Container<'a> = &'a mut FieldTransforms;

    #[inline]
    fn reborrow<'a, 'new>(container: &'new mut Self::Container<'a>) -> Self::Container<'new> {
        container
    }

    #[inline]
    fn server_timestamp() -> Result<Self, SerError> {
        Ok(ServerTimestamp::TRANSFORM)
    }

    #[inline]
    fn increment<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, SerError> {
        serialize_numeric(value, "increment").map(TransformType::Increment)
    }

    #[inline]
    fn minimum<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, SerError> {
        serialize_numeric(value, "minimum").map(TransformType::Minimum)
    }

    #[inline]
    fn maximum<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, SerError> {
        serialize_numeric(value, "maximum").map(TransformType::Maximum)
    }

    #[inline]
    fn remove_all_from_array<T: serde::Serialize + ?Sized, W: WriteKind>(
        value: &T,
    ) -> Result<Self, SerError> {
        serialize_array::<T, W>(value, "remove_all_from_array")
            .map(TransformType::RemoveAllFromArray)
    }

    #[inline]
    fn append_missing_elements<T: serde::Serialize + ?Sized, W: WriteKind>(
        value: &T,
    ) -> Result<Self, SerError> {
        serialize_array::<T, W>(value, "append_missing_elements")
            .map(TransformType::AppendMissingElements)
    }
}

fn serialize_numeric<T: serde::Serialize + ?Sized>(
    value: &T,
    kind: &'static str,
) -> Result<firestore::Value, SerError> {
    use firestore::value::ValueType::{DoubleValue, IntegerValue};

    match super::serialize_value::<super::Update>(value)? {
        numeric @ (DoubleValue(_) | IntegerValue(_)) => Ok(firestore::Value {
            value_type: Some(numeric),
        }),
        other => Err(SerError::InvalidTransform(InvalidTransform::new(
            kind, other,
        ))),
    }
}

fn serialize_array<T: serde::Serialize + ?Sized, W: WriteKind>(
    value: &T,
    kind: &'static str,
) -> Result<firestore::ArrayValue, SerError> {
    match super::serialize_value::<W>(value)? {
        firestore::value::ValueType::ArrayValue(array) => Ok(array),
        other => Err(SerError::InvalidTransform(InvalidTransform::new(
            kind, other,
        ))),
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServerTimestamp;

impl ServerTimestamp {
    pub(crate) const MARKER: &str = "__field_transform_server_timestamp__";

    pub(crate) const TRANSFORM: TransformType =
        TransformType::SetToServerValue(ServerValue::RequestTime as i32);
}

impl serde::Serialize for ServerTimestamp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_struct(Self::MARKER)
    }
}

impl<'de> serde::Deserialize<'de> for ServerTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(serde::de::IgnoredAny)?;
        Ok(Self)
    }
}

macro_rules! impl_numeric_transform_wrapper {
    (
        $($name:ident[$lowercase:literal]),* $(,)?
    ) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name<T>(pub T);

            impl $name<()> {
                pub(crate) const MARKER: &str = concat!("__field_transform_", $lowercase, "__");

                #[doc = concat!("Serializes any [`T`] as [`", stringify!($name), "<T>`]")]
                /// Intended to be used with serde
                /// derive attributes, ex:
                /// ```
                #[doc = concat!("use firestore_rs::transform::", stringify!($name), ";")]
                ///
                /// #[derive(serde::Serialize)]
                /// struct Document {
                #[doc = concat!("     #[serde(serialize_with = \"", stringify!($name), "::serialize\")]")]
                ///     value: u32,
                /// }
                /// ```
                #[inline]
                pub fn serialize<T, S>(inner: &T, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                    T: serde::Serialize,
                {
                    serde::Serialize::serialize(&$name(inner), serializer)
                }
            }

            impl<T: serde::Serialize> serde::Serialize for $name<T> {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct($name::<()>::MARKER, &self.0)
                }
            }

            impl<'de, T> serde::Deserialize<'de> for $name<T>
            where
                T: serde::Deserialize<'de>,
            {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    T::deserialize(deserializer).map(Self)
                }
            }
        )*
    };
}

impl_numeric_transform_wrapper! {
    Increment["increment"],
    Minimum["minimum"],
    Maximum["maximum"],
}

macro_rules! impl_array_transform_wrapper {
    (
        $($name:ident[$lowercase:literal]),* $(,)?
    ) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name<T>(pub T);

            impl $name<()> {
                pub(crate) const MARKER: &str = concat!("__field_transform_", $lowercase, "__");

                #[doc = concat!("Serializes any [`T`] as [`", stringify!($name), "<T>`]")]
                /// Intended to be used with serde
                /// derive attributes, ex:
                /// ```
                #[doc = concat!("use firestore_rs::transform::", stringify!($name), ";")]
                ///
                /// #[derive(serde::Serialize)]
                /// struct Document {
                #[doc = concat!("     #[serde(serialize_with = \"", stringify!($name), "::serialize\")]")]
                ///     value: u32,
                /// }
                /// ```
                #[inline]
                pub fn serialize<T, S>(inner: &T, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                    T: serde::Serialize,
                {
                    serde::Serialize::serialize(&$name(inner), serializer)
                }
            }

            impl<T: serde::Serialize> serde::Serialize for $name<T> {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct($name::<()>::MARKER, &self.0)
                }
            }

            impl<'de, T> serde::Deserialize<'de> for $name<T>
            where
                T: serde::Deserialize<'de>,
            {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    T::deserialize(deserializer).map(Self)
                }
            }
        )*
    };
}

impl_array_transform_wrapper! {
    RemoveFromArray["remove_from_array"],
    ArrayUnion["array_union"],
}
