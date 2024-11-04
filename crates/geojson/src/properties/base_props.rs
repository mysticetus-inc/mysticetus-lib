//! Contains the [`Properties`] trait, and the default implementor, [`BaseProperties`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use timestamp::Timestamp;
use uuid::Uuid;

use super::{DisplayProps, InnerPropertyMap, Properties};
use crate::builder::BasePropertiesBuilder;
use crate::private::Sealed;

pub trait Requirement<T>: Sealed {
    const REQUIRED: bool;

    type AsRefType<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref_type(&self) -> Self::AsRefType<'_>;

    fn should_skip_serializing(&self) -> bool;
}

impl Requirement<Timestamp> for Timestamp {
    const REQUIRED: bool = true;

    fn as_ref_type(&self) -> Self::AsRefType<'_> {
        self
    }

    fn should_skip_serializing(&self) -> bool {
        false
    }
}

impl Requirement<Timestamp> for Option<Timestamp> {
    const REQUIRED: bool = false;

    type AsRefType<'a> = Option<&'a Timestamp>;

    fn as_ref_type(&self) -> Self::AsRefType<'_> {
        self.as_ref()
    }

    fn should_skip_serializing(&self) -> bool {
        self.is_some()
    }
}

impl Requirement<String> for String {
    const REQUIRED: bool = true;

    type AsRefType<'a> = &'a str;

    fn as_ref_type(&self) -> Self::AsRefType<'_> {
        self.as_str()
    }

    fn should_skip_serializing(&self) -> bool {
        self.is_empty()
    }
}

impl Requirement<String> for Option<String> {
    const REQUIRED: bool = false;

    type AsRefType<'a> = Option<&'a str>;

    fn as_ref_type(&self) -> Self::AsRefType<'_> {
        self.as_deref()
    }

    fn should_skip_serializing(&self) -> bool {
        match self.as_ref() {
            Some(string) => string.is_empty(),
            None => true,
        }
    }
}

pub trait IntoBaseProperties<Id = Uuid, Ts = Timestamp, Name = String, Val = serde_json::Value>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    fn into_base_props(self) -> BaseProperties<Id, Ts, Name, Val>;
}

pub trait TryIntoBaseProperties<Id = Uuid, Ts = Timestamp, Name = String, Val = serde_json::Value>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    type Error;
    fn try_into_base_props(self) -> Result<BaseProperties<Id, Ts, Name, Val>, Self::Error>;
}

impl<T, Id, Ts, Name, Val> TryIntoBaseProperties<Id, Ts, Name, Val> for T
where
    T: IntoBaseProperties<Id, Ts, Name, Val>,
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    type Error = std::convert::Infallible;

    fn try_into_base_props(self) -> Result<BaseProperties<Id, Ts, Name, Val>, Self::Error> {
        Ok(self.into_base_props())
    }
}

impl<Id, Ts, Name, Val> IntoBaseProperties<Id, Ts, Name, Val> for BaseProperties<Id, Ts, Name, Val>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    fn into_base_props(self) -> BaseProperties<Id, Ts, Name, Val> {
        self
    }
}

/// A generic implementor of [`Properties`], with a generic value type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct BaseProperties<Id = Uuid, Ts = Timestamp, Name = String, Val = serde_json::Value>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    pub id: Id,
    #[serde(skip_serializing_if = "Requirement::should_skip_serializing")]
    pub epoch: Ts,
    #[serde(skip_serializing_if = "Requirement::should_skip_serializing")]
    pub name: Name,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub display_properties: BTreeMap<String, Val>,
    #[serde(flatten)]
    pub props: BTreeMap<String, Val>,
}

impl<Id, Ts, Name, Val> From<BasePropertiesBuilder<Id, Ts, Name, Val>>
    for BaseProperties<Id, Ts, Name, Val>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    fn from(builder: BasePropertiesBuilder<Id, Ts, Name, Val>) -> Self {
        Self {
            id: builder.id,
            name: builder.name,
            epoch: builder.epoch,
            display_properties: builder.display_properties,
            props: builder.props,
        }
    }
}

impl<Val> BaseProperties<(), Option<Timestamp>, Option<String>, Val> {
    pub fn builder() -> BasePropertiesBuilder<(), Option<Timestamp>, Option<String>, Val> {
        BasePropertiesBuilder::new()
    }
}

impl<Id, Val> BaseProperties<Id, Option<Timestamp>, Option<String>, Val> {
    /// Instantiates the properties with a given Id.
    pub fn new_with_id(id: Id) -> Self {
        Self {
            id,
            name: None,
            epoch: None,
            display_properties: BTreeMap::new(),
            props: BTreeMap::new(),
        }
    }
}

impl<Id, Ts, Name, Val> BaseProperties<Id, Ts, Name, Val>
where
    Id: Default,
    Ts: Requirement<Timestamp> + Default,
    Name: Requirement<String> + Default,
{
    /// Instantiates the properties with a randomly generated Id.
    ///
    /// [`Default`] calls this under the hood.
    pub fn new() -> Self {
        Self {
            id: Id::default(),
            name: Name::default(),
            epoch: Ts::default(),
            display_properties: BTreeMap::new(),
            props: BTreeMap::new(),
        }
    }
}

impl<Ts, Name, Val> BaseProperties<Uuid, Ts, Name, Val>
where
    Ts: Requirement<Timestamp> + Default,
    Name: Requirement<String> + AsRef<[u8]>,
{
    pub fn new_id_from_name(name: Name) -> Self {
        Self {
            id: Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_ref()),
            name,
            epoch: Ts::default(),
            display_properties: BTreeMap::new(),
            props: BTreeMap::new(),
        }
    }
}

impl<Id, Ts, Name, Val> Default for BaseProperties<Id, Ts, Name, Val>
where
    Id: Default,
    Ts: Requirement<Timestamp> + Default,
    Name: Requirement<String> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, Ts, Name, V> InnerPropertyMap for BaseProperties<Id, Ts, Name, V>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    type Map = BTreeMap<String, V>;

    fn property_map(&self) -> &Self::Map {
        &self.props
    }

    fn property_map_mut(&mut self) -> &mut Self::Map {
        &mut self.props
    }

    fn into_property_map(self) -> Self::Map {
        self.props
    }
}

impl<Id, Ts, Name, V> Properties for BaseProperties<Id, Ts, Name, V>
where
    Id: Copy + std::hash::Hash + Ord,
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
    for<'de> V: Deserialize<'de> + Serialize,
{
    type Id = Id;

    type RequiredArgs = (Id, Name, Ts);

    type Name = Name;
    type NameRef<'a>
        = <Name as Requirement<String>>::AsRefType<'a>
    where
        Self: 'a;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn new(args: Self::RequiredArgs) -> Self {
        Self {
            id: args.0,
            name: args.1,
            epoch: args.2,
            display_properties: BTreeMap::new(),
            props: BTreeMap::new(),
        }
    }

    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }

    fn name(&self) -> Self::NameRef<'_> {
        self.name.as_ref_type()
    }

    fn name_mut(&mut self) -> &mut Self::Name {
        &mut self.name
    }
}

impl<Id, Ts, Name, V> DisplayProps for BaseProperties<Id, Ts, Name, V>
where
    Id: Copy + std::hash::Hash + Ord,
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
    for<'de> V: Deserialize<'de> + Serialize,
{
    type DisplayProps = BTreeMap<String, V>;

    fn display_props(&self) -> &Self::DisplayProps {
        &self.display_properties
    }

    fn display_props_mut(&mut self) -> &mut Self::DisplayProps {
        &mut self.display_properties
    }
}

impl<Id, Name, Val> super::TimedProps for BaseProperties<Id, Timestamp, Name, Val>
where
    Id: Copy + std::hash::Hash + Ord,
    Name: Requirement<String>,
    for<'de> Val: Deserialize<'de> + Serialize,
{
    fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.epoch = timestamp;
    }

    fn timestamp(&self) -> Timestamp {
        self.epoch
    }
}
