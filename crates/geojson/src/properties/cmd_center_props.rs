//! Command center specific property definitions.
use std::fmt;

use serde::{Deserialize, Serialize, de, ser};

// use crate::builder::CmdCenterPropsBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DataType {
    Sighting,
    Station,
    Glider,
    Buoy,
    LeaseArea,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sighting => Self::SIGHTING.as_str(),
            Self::Station => Self::STATION.as_str(),
            Self::LeaseArea => Self::LEASE_AREA.as_str(),
            Self::Glider => Self::GLIDER.as_str(),
            Self::Buoy => Self::BUOY.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MismatchedDataType<T>
where
    T: AsRef<str> + crate::private::Sealed,
{
    found: DataType,
    expected: T,
}

impl<T> fmt::Display for MismatchedDataType<T>
where
    T: AsRef<str> + crate::private::Sealed,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "mismatched data type string, found '{}', but expected '{}'",
            self.found.as_str(),
            self.expected.as_ref(),
        )
    }
}

macro_rules! impl_data_types {
    ($($name:ident, $const_name:ident, $string:literal),* $(,)?) => {
        $(
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
            pub struct $name;

            impl crate::private::Sealed for $name {}

            impl $name {
                pub fn as_str(&self) -> &'static str {
                    $string
                }
            }

            impl From<$name> for DataType {
                fn from(_: $name) -> Self {
                    Self::$name
                }
            }

            impl TryFrom<DataType> for $name {
                type Error = MismatchedDataType<$name>;

                fn try_from(data_type: DataType) -> Result<Self, Self::Error> {
                    match data_type {
                        DataType::$name => Ok($name),
                        found => Err(MismatchedDataType { found, expected: $name }),
                    }
                }
            }

            impl AsRef<str> for $name {
                fn as_ref(&self) -> &str {
                    self.as_str()
                }
            }

            impl DataType {
                pub const $const_name: $name = $name;
            }

            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ser::Serializer
                {
                    serializer.serialize_str(self.as_str())
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>
                {
                    deserializer.deserialize_str($name)
                }
            }

            impl<'de> de::Visitor<'de> for $name {
                type Value = Self;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "a data type string, equal to '{}'", self.as_str())
                }

                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: de::Error
                {
                    if s.trim() == self.as_str() {
                        return Ok(Self)
                    }

                    Err(de::Error::invalid_value(de::Unexpected::Str(s), &self))
                }
            }
        )*
    };
}

impl_data_types! {
    Sighting, SIGHTING, "sighting",
    Station, STATION, "station",
    LeaseArea, LEASE_AREA, "leaseArea",
    Glider, GLIDER, "glider",
    Buoy, BUOY, "buoy",
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdCenterProps<'a, P>(&'a P);

/*
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdCenterProps<Val> {
    #[serde(skip_serializing_if = "Option::is_none")]
    client: Option<String>,
    #[serde(rename = "epoch", skip_serializing_if = "Option::is_none")]
    timestamp: Option<Timestamp>,
    data_type: DataType,
    #[serde(flatten)]
    base: BaseProps<Val>,
}


impl<Val> CmdCenterProps<Val> {
    pub fn client(&self) -> Option<&str> {
        self.client.as_deref()
    }
}

impl<Val> InnerPropertyMap for CmdCenterProps<Val> {
    type Map = <BaseProps<Val> as InnerPropertyMap>::Map;

    fn property_map(&self) -> &Self::Map {
        self.base.property_map()
    }

    fn property_map_mut(&mut self) -> &mut Self::Map {
        self.base.property_map_mut()
    }

    fn into_property_map(self) -> Self::Map {
        self.base.into_property_map()
    }
}



impl<Val, C, T, DT> From<CmdCenterPropsBuilder<uuid::Uuid, Val, C, T, DT>> for CmdCenterProps<Val>
where
    T: Into<Option<Timestamp>>,
    C: Into<Option<String>>,
    DT: Into<DataType>
{
    fn from(builder: CmdCenterPropsBuilder<uuid::Uuid, Val, C, T, DT>) -> Self {
        Self {
            client: builder.client.into(),
            timestamp: builder.timestamp.into(),
            data_type: builder.data_type.into(),
            base: builder.base.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SightingProps<Val> {
    client: String,
    #[serde(rename = "epoch")]
    timestamp: Timestamp,
    data_type: Sighting,
    #[serde(flatten)]
    base: BaseProps<Val>,
}

impl<Val> SightingProps<Val> {
    pub fn client(&self) -> &str {
        self.client.as_str()
    }
}

impl<Val> InnerPropertyMap for SightingProps<Val> {
    type Map = <BaseProps<Val> as InnerPropertyMap>::Map;

    fn property_map(&self) -> &Self::Map {
        self.base.property_map()
    }

    fn property_map_mut(&mut self) -> &mut Self::Map {
        self.base.property_map_mut()
    }

    fn into_property_map(self) -> Self::Map {
        self.base.into_property_map()
    }
}

impl<Val, DT> From<CmdCenterPropsBuilder<uuid::Uuid, Val, String, Timestamp, DT>>
for SightingProps<Val> {
    fn from(
        builder: CmdCenterPropsBuilder<uuid::Uuid, Val, String, Timestamp, DT>
    ) -> Self {
        Self {
            client: builder.client,
            timestamp: builder.timestamp,
            data_type: Sighting,
            base: builder.base.into(),
        }
    }
}


macro_rules! impl_generic_property_fns {
    ($properties_field:ident) => {
        fn id(&self) -> Self::Id {
            self.$properties_field.id()
        }

        fn set_id(&mut self, id: Self::Id) {
            self.$properties_field.set_id(id);
        }

        fn name(&self) -> Self::NameRef<'_> {
            self.$properties_field.name()
        }

        fn name_mut(&mut self) -> &mut Self::Name {
            self.$properties_field.name_mut()
        }
    };
}

macro_rules! impl_disp_props {
    ($($type:ident),* $(,)?) => {
        $(
            impl<Val> DisplayProps for $type<Val>
            where
                for<'de> Val: Deserialize<'de> + Serialize
            {
                type DisplayProps = <BaseProps<Val> as DisplayProps>::DisplayProps;

                fn display_props(&self) -> &Self::DisplayProps {
                    self.base.display_props()
                }

                fn display_props_mut(&mut self) -> &mut Self::DisplayProps {
                    self.base.display_props_mut()
                }
            }
        )*
    };
}

impl_disp_props!(CmdCenterProps, SightingProps);


impl<Val> Properties for CmdCenterProps<Val>
where
    for<'de> Val: Deserialize<'de> + Serialize,
{
    type Id = uuid::Uuid;

    type RequiredArgs = (
        Option<Self::Id>,
        Option<String>,
        DataType,
        Option<Timestamp>,
    );

    type Name = <BaseProps<Val> as Properties>::Name;

    type NameRef<'a> = <BaseProps<Val> as Properties>::NameRef<'a>
    where
        Self: 'a;

    fn new(args: Self::RequiredArgs) -> Self {
        Self {
            base: BaseProps::new_with_id(args.0.unwrap_or_else(uuid::Uuid::new_v4)),
            client: args.1,
            data_type: args.2,
            timestamp: args.3,
        }
    }

    impl_generic_property_fns!(base);
}

impl<Val> CmdCenterProps<Val> {
    /// Builds a new instance of [`CmdCenterProps`] with a known `DataType`.
    pub fn new_with_data_type(data_type: DataType) -> Self {
        Self {
            client: None,
            timestamp: None,
            data_type,
            base: BaseProps::new(),
        }
    }

    /// Sets the inner data_type, returning the existing data_type that was replaced.
    pub fn set_data_type(&mut self, data_type: DataType) -> DataType {
        std::mem::replace(&mut self.data_type, data_type)
    }

    /// Returns a reference to the data type.
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Sets the timestamp, returning the one that was optionally already set.
    pub fn set_timestamp(&mut self, timestamp: Timestamp) -> Option<Timestamp> {
        self.timestamp.replace(timestamp)
    }

    /// Gets the optional timestamp.
    pub fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp
    }

    /// Removes the timestamp, returning the one that was previously set.
    pub fn clear_timestamp(&mut self) -> Option<Timestamp> {
        self.timestamp.take()
    }

    /// Attempts to convert [`Self`] into a [`TimedProps`]. If the inner timestamp is [`Some`],
    /// the [`Ok`] variant will be returned. If the inner timestamp is [`None`], returns the
    /// unmodified [`Self`] as the [`Err`] variant.
    pub fn into_sighting_props(mut self) -> Result<SightingProps<Val>, Self> {
        match (self.data_type, self.client.take(), self.timestamp) {
            (DataType::Sighting, Some(client), Some(timestamp)) => {
                Ok(SightingProps {
                    timestamp,
                    client,
                    data_type: Sighting,
                    base: self.base,
                })
            },
            (_, client, _) => {
                // since we took client in the match statement to avoid moving out of 'self', we
                // need to re-set it before returning.
                self.client = client;
                Err(self)
            }
        }
    }

    pub fn into_sighting_props_with<F>(self, fallback: F) -> Result<SightingProps<Val>, Self>
    where
        F: FnOnce() -> (Timestamp, String),
    {
        if self.data_type != DataType::Sighting {
            return Err(self);
        }

        let (timestamp, client) = self.timestamp.zip(self.client).unwrap_or_else(fallback);

        Ok(SightingProps {
            base: self.base,
            timestamp,
            client,
            data_type: Sighting,
        })
    }
}

impl<Val> SightingProps<Val> {
    pub fn new_now<S>(client: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            base: BaseProps::new(),
            data_type: Sighting,
            timestamp: Timestamp::now(),
            client: client.into(),
        }
    }

    pub fn new<S>(client: S, timestamp: Timestamp) -> Self
    where
        S: Into<String>,
    {
        Self {
            base: BaseProps::new(),
            data_type: Sighting,
            timestamp,
            client: client.into(),
        }
    }

    pub fn new_with_id<S>(client: S, id: uuid::Uuid, timestamp: Timestamp) -> Self
    where
        S: Into<String>,
    {
        Self {
            base: BaseProps::new_with_id(id),
            data_type: Sighting,
            timestamp,
            client: client.into(),
        }
    }

    pub fn into_general_props(self) -> CmdCenterProps<Val> {
        CmdCenterProps {
            base: self.base,
            data_type: DataType::Sighting,
            timestamp: Some(self.timestamp),
            client: Some(self.client),
        }
    }
}

impl<Val> Properties for SightingProps<Val>
where
    for<'de> Val: Deserialize<'de> + Serialize,
{
    type Id = uuid::Uuid;

    type RequiredArgs = (Option<Self::Id>, String, Option<Timestamp>);

    type Name = <BaseProps<Val> as Properties>::Name;

    type NameRef<'a> = <BaseProps<Val> as Properties>::NameRef<'a>
    where
        Self: 'a;

    fn new(args: Self::RequiredArgs) -> Self {
        Self {
            base: BaseProps::new_with_id(args.0.unwrap_or_else(uuid::Uuid::new_v4)),
            client: args.1,
            timestamp: args.2.unwrap_or_else(Timestamp::now),
            data_type: Sighting,
        }
    }

    impl_generic_property_fns!(base);
}


impl<Val> TimedProps for SightingProps<Val>
where
    for<'de> Val: Deserialize<'de> + Serialize,
{
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamp = timestamp;
    }
}
*/
