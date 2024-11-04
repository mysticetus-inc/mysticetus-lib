#[cfg(feature = "protobuf")]
mod protobuf;
#[cfg(feature = "rpc")]
mod rpc;

/*
#[cfg(any(feature = "mysticetus", feature = "mysticetus-video"))]
mod date;
*/
#[cfg(feature = "protobuf")]
pub(crate) mod duration;
#[cfg(feature = "protobuf")]
pub(crate) mod timestamp;

// use type_mods::EnumStr;
/*

pub trait ProstEnumHelper: Sized + private::Sealed {
    fn from_i32(i: i32) -> Option<Self>;

    fn to_i32(self) -> i32;

    fn from_opt_i32(i: Option<i32>) -> Option<Self> {
        i.and_then(Self::from_i32)
    }
}

mod private {
    pub trait Sealed {}
}

macro_rules! impl_enum_helper {
    ($(
        $feature:literal => { $($type:ty),* $(,)? }

    ),* $(,)?) => {
        $(
            $(
                #[cfg(feature = $feature)]
                impl private::Sealed for $type { }

                #[cfg(feature = $feature)]
                impl ProstEnumHelper for $type {
                    fn from_i32(i: i32) -> Option<Self> {
                        Self::from_i32(i)
                    }

                    fn to_i32(self) -> i32 {
                        self as i32
                    }
                }
            )*
        )*
    };
}

impl_enum_helper! {
    "mysticetus-video" => {
        crate::protos::mysticetus::video::CameraType,
        crate::protos::mysticetus::video::DetectionType,
        crate::protos::mysticetus::video::BirdSpecies,
        crate::protos::mysticetus::video::VideoLabelingStatus,
        crate::protos::mysticetus::common::Species,
    }
}

pub struct SerdeEnumStr<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> SerdeEnumStr<T>
where
    T: ProstEnumHelper + EnumStr,
{
    pub fn serialize<S>(t: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match T::from_i32(*t) {
            Some(v) => serializer.collect_str(v.as_enum_str()),
            None => serializer.serialize_i32(*t),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match deserializer.deserialize_any(Visitor(std::marker::PhantomData::<T>)) {
            Ok(v) => Ok(v.to_i32()),
            Err(e) => Err(e),
        }
    }
}

struct Visitor<T>(std::marker::PhantomData<T>);

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: ProstEnumHelper + EnumStr,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string or int instance of '{}'",
            std::any::type_name::<T>()
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match T::try_from_enum_str(v) {
            Some(p) => Ok(p),
            None => Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i32(v as i32)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match T::from_i32(v) {
            Some(v) => Ok(v),
            None => Err(de::Error::invalid_value(
                de::Unexpected::Signed(v as _),
                &self,
            )),
        }
    }
}

pub struct SerdeEnumStrOpt<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> SerdeEnumStrOpt<T>
where
    T: ProstEnumHelper + EnumStr,
{
    pub fn serialize<S>(t: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let i = match t {
            Some(i) => i,
            None => return serializer.serialize_none(),
        };

        match T::from_i32(*i) {
            Some(v) => serializer.serialize_some(v.as_enum_str()),
            None => serializer.serialize_some(i),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vis = OptionVisitor(Visitor(std::marker::PhantomData::<T>));
        match deserializer.deserialize_option(vis) {
            Ok(Some(v)) => Ok(Some(v.to_i32())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

struct OptionVisitor<T>(Visitor<T>);

impl<'de, T> de::Visitor<'de> for OptionVisitor<T>
where
    T: ProstEnumHelper,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "an optional string or int instance of '{}'",
            std::any::type_name::<T>()
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let t = v.trim();

        match T::try_from_enum_str(t) {
            Some(t) => Ok(Some(t)),
            None if t.is_empty() => Ok(None),
            _ => Err(de::Error::invalid_value(de::Unexpected::Str(t), &self)),
        }
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.visit_some(deserializer).map(Some)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.visit_newtype_struct(deserializer).map(Some)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i32(v as i32)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match T::from_i32(v) {
            Some(v) => Ok(Some(v)),
            None if v == 0 => Ok(None),
            None => Err(de::Error::invalid_value(
                de::Unexpected::Signed(v as _),
                &self,
            )),
        }
    }
}
*/
