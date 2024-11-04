//! Builders for assembling [`BaseProperties`] and derived types.

use std::collections::BTreeMap;

// use crate::properties::cmd_center_props::{DataType, CmdCenterProps, SightingProps};
use timestamp::Timestamp;

use crate::properties::base_props::{BaseProperties, Requirement};

/// A builder pattern struct for constructing [`BaseProperties`].
///
/// Useful as an inner builder for types that contain [`BaseProperties`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasePropertiesBuilder<Id, Ts, Name, Val> {
    pub(crate) id: Id,
    pub(crate) name: Name,
    pub(crate) epoch: Ts,
    pub(crate) display_properties: BTreeMap<String, Val>,
    pub(crate) props: BTreeMap<String, Val>,
}

impl<Val> BasePropertiesBuilder<(), Option<Timestamp>, Option<String>, Val> {
    /// Builds a new [`BasePropertiesBuilder`] with an optional `Id`, set to [`None`].
    pub fn new() -> Self {
        BasePropertiesBuilder {
            id: (),
            name: None,
            epoch: None,
            display_properties: BTreeMap::new(),
            props: BTreeMap::new(),
        }
    }
}

impl<Val> Default for BasePropertiesBuilder<(), Option<Timestamp>, Option<String>, Val> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, Val> BasePropertiesBuilder<Id, Option<Timestamp>, Option<String>, Val> {
    /// Builds a new [`BasePropertiesBuilder`] with a known `Id`.
    pub fn new_with_id(id: Id) -> Self {
        BasePropertiesBuilder::new().set_id(id)
    }
}

impl<Id, Ts, Name> BasePropertiesBuilder<Id, Ts, Name, serde_json::Value> {
    /// Inserts a property that is serializable into [`serde_json::Value`].
    pub fn serialize_property<S, V>(
        mut self,
        key: S,
        value: V,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        S: Into<String>,
        V: serde::Serialize,
    {
        match serde_json::to_value(value) {
            Ok(value) => {
                self.props.insert(key.into(), value);
                Ok(self)
            }
            Err(err) => Err((self, err)),
        }
    }

    /// Serializes many properties from an iterator over key-value pairs.
    pub fn serialize_properties<I, S, V>(
        mut self,
        iter: I,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: serde::Serialize,
    {
        for (key, value) in iter.into_iter() {
            self = self.serialize_property(key, value)?;
        }

        Ok(self)
    }

    /// Inserts a display property that is serializable into [`serde_json::Value`].
    pub fn serialize_display_property<S, V>(
        mut self,
        key: S,
        value: V,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        S: Into<String>,
        V: serde::Serialize,
    {
        match serde_json::to_value(value) {
            Ok(value) => {
                self.display_properties.insert(key.into(), value);
                Ok(self)
            }
            Err(err) => Err((self, err)),
        }
    }

    /// Serializes many display properties from an iterator over key-value pairs.
    pub fn serialize_display_properties_properties<I, S, V>(
        mut self,
        iter: I,
    ) -> Result<Self, (Self, serde_json::Error)>
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: serde::Serialize,
    {
        for (key, value) in iter.into_iter() {
            self = self.serialize_display_property(key, value)?;
        }

        Ok(self)
    }
}

impl<Id, Ts, Name, Val> BasePropertiesBuilder<Id, Ts, Name, Val> {
    /// Inserts a property into the underlying map.
    pub fn insert_property<S, V>(mut self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<Val>,
    {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Inserts a property into the map, if it is [`Some`]. Convience function to avoid needing
    /// tons of `if let Some(...) = ... {}` blocks and re-setting a mut variable for this builder.
    pub fn insert_property_if_some<S, V>(self, key: S, value_opt: Option<V>) -> Self
    where
        S: Into<String>,
        V: Into<Val>,
    {
        match value_opt {
            Some(value) => self.insert_property(key.into(), value.into()),
            None => self,
        }
    }

    /// Extends the underlying property map with an iterator of key value pairs.
    pub fn extend_properties<I, S, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<Val>,
    {
        self.props.extend(
            iter.into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    /// Inserts a display property into the underlying map.
    pub fn insert_display_property<S, V>(mut self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<Val>,
    {
        self.display_properties.insert(key.into(), value.into());
        self
    }

    /// Inserts a display property into the map, if it is [`Some`]. Convience function to avoid
    /// needing tons of `if let Some(...) = ... {}` blocks and re-setting a mut variable for this
    /// builder.
    pub fn insert_display_property_if_some<S, V>(self, key: S, value_opt: Option<V>) -> Self
    where
        S: Into<String>,
        V: Into<Val>,
    {
        match value_opt {
            Some(value) => self.insert_display_property(key.into(), value.into()),
            None => self,
        }
    }

    /// Extends the display properties from an iterator of key value pairs.
    pub fn extend_display_properties<I, S, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<Val>,
    {
        self.display_properties.extend(
            iter.into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }
}

impl<Id, Ts, Val> BasePropertiesBuilder<Id, Ts, Option<String>, Val> {
    /// Sets the `name` field, but leaves the type as optional.
    pub fn set_name<S>(self, name: S) -> Self
    where
        S: Into<String>,
    {
        BasePropertiesBuilder {
            id: self.id,
            name: Some(name.into()),
            epoch: self.epoch,
            display_properties: self.display_properties,
            props: self.props,
        }
    }

    /// Sets the `name` field, and change the type of name from [`Option<String>`] -> [`String`]
    pub fn set_required_name<S>(self, name: S) -> BasePropertiesBuilder<Id, Ts, String, Val>
    where
        S: Into<String>,
    {
        BasePropertiesBuilder {
            id: self.id,
            name: name.into(),
            epoch: self.epoch,
            display_properties: self.display_properties,
            props: self.props,
        }
    }
}

impl<Ts, Name, Val> BasePropertiesBuilder<(), Ts, Name, Val> {
    /// Sets the `id` field.
    pub fn set_id<Id>(self, id: Id) -> BasePropertiesBuilder<Id, Ts, Name, Val> {
        BasePropertiesBuilder {
            id,
            name: self.name,
            epoch: self.epoch,
            display_properties: self.display_properties,
            props: self.props,
        }
    }
}

impl<Id, Name, Val> BasePropertiesBuilder<Id, Option<Timestamp>, Name, Val> {
    /// Sets the `epoch/timestamp` field, but leaves the timestamp type optional.
    pub fn set_timestamp(self, timestamp: Timestamp) -> Self {
        Self {
            id: self.id,
            name: self.name,
            epoch: Some(timestamp),
            display_properties: self.display_properties,
            props: self.props,
        }
    }
    /// Sets the `epoch/timestamp` field, but changes types to make it a required field.
    pub fn set_required_timestamp(
        self,
        timestamp: Timestamp,
    ) -> BasePropertiesBuilder<Id, Timestamp, Name, Val> {
        BasePropertiesBuilder {
            id: self.id,
            name: self.name,
            epoch: timestamp,
            display_properties: self.display_properties,
            props: self.props,
        }
    }
}

impl<Id, Ts, Name, Val> BasePropertiesBuilder<Id, Ts, Name, Val>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    /// Build the [`BaseProperties`].
    pub fn build(self) -> BaseProperties<Id, Ts, Name, Val> {
        self.into()
    }
}

impl<Id, Ts, Name, Val> BasePropertiesBuilder<Option<Id>, Ts, Name, Val>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    /// Sets the `Id`, returning a new instance of the builder with a non-[`Option`] `Id`.
    pub fn set_id(self, id: Id) -> BasePropertiesBuilder<Id, Ts, Name, Val> {
        BasePropertiesBuilder {
            id,
            name: self.name,
            epoch: self.epoch,
            display_properties: self.display_properties,
            props: self.props,
        }
    }
}

impl<Id, Name, Val> BasePropertiesBuilder<Id, Option<Timestamp>, Name, Val>
where
    Name: Requirement<String>,
{
    /// Sets the inner, optional timestamp, overwriting any previous timestamp. If the goal is to
    /// insert the timestamp as a required type, use [`Self::with_required_timestamp`] to change
    /// the type.
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.epoch = Some(timestamp);
        self
    }

    /// Inserts a timestamp, changing the type of the timestamp field on the builder from
    /// [`Option<Timestamp>`] -> [`Timestamp`]
    pub fn with_required_timestamp(
        self,
        timestamp: Timestamp,
    ) -> BasePropertiesBuilder<Id, Timestamp, Name, Val> {
        BasePropertiesBuilder {
            id: self.id,
            name: self.name,
            epoch: timestamp,
            display_properties: self.display_properties,
            props: self.props,
        }
    }
}

impl<Id, Ts, Name, Val> BasePropertiesBuilder<Id, Ts, Name, Val>
where
    Ts: Requirement<Timestamp>,
    Name: Requirement<String>,
{
    /// Overwrites the current `Id` with a new one.
    pub fn overwrite_id(mut self, new_id: Id) -> Self {
        self.id = new_id;
        self
    }
}

/*

/// A builder pattern object for constructing [`CmdCenterProps`] and its variants (i.e
/// [`SightingProps`])
#[derive(Debug, Clone, PartialEq)]
pub struct CmdCenterPropsBuilder<Id, Val, C, T, DT> {
    pub(crate) base: BasePropertiesBuilder<Id, T, Val>,
    pub(crate) client: C,
    pub(crate) data_type: DT,
}

impl<Id, Val, C, T, DT> CmdCenterPropsBuilder<Id, Val, C, T, DT> {
    /// Builds a new [`CmdCenterPropsBuilder`] with an optional `Id`, `client`, `timestamp` and
    /// `data_type`, all set to [`None`].
    pub fn new() -> CmdCenterPropsBuilder<Option<Id>, Val, Option<C>, Option<T>, Option<DT>> {
        CmdCenterPropsBuilder {
            base: BasePropertiesBuilder::new(),
            client: None,
            data_type: None,
        }
    }

    /// Builds a new [`CmdCenterPropsBuilder`] with a known `Id`. The `client`, `timestamp` and
    /// `data_type` fields are all set to [`None`].
    pub fn new_with_id(id: Id) -> CmdCenterPropsBuilder<Id, Val, Option<C>, Option<T>, Option<DT>> {
        CmdCenterPropsBuilder {
            base: BasePropertiesBuilder::new_with_id(id),
            client: None,
            data_type: None,
        }
    }

    /// Inserts a property into the underlying map.
    pub fn insert_property<S, V>(mut self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<Val>
    {
        self.base.props.insert(key.into(), value.into());
        self
    }

    /// Inserts a property into the map, if it is [`Some`]. Convience function to avoid needing
    /// tons of `if let Some(...) = ... {}` blocks and re-setting a mut variable for this builder.
    pub fn insert_property_if_some<S, V>(self, key: S, value_opt: Option<V>) -> Self
    where
        S: Into<String>,
        V: Into<Val>,
    {
        match value_opt {
            Some(value) => self.insert_property(key.into(), value.into()),
            None => self,
        }
    }

    /// Extends the underlying property map with an iterator of key value pairs.
    pub fn extend_properties<I, S, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<Val>
    {
        self.base.props.extend(iter.into_iter().map(|(key, value)| (key.into(), value.into())));
        self
    }

    /// Inserts a display property into the underlying map.
    pub fn insert_display_property<S, V>(mut self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<Val>
    {
        self.base.display_properties.insert(key.into(), value.into());
        self
    }

    /// Inserts a display property into the map, if it is [`Some`]. Convience function to avoid
    /// needing tons of `if let Some(...) = ... {}` blocks and re-setting a mut variable for this
    /// builder.
    pub fn insert_display_property_if_some<S, V>(self, key: S, value_opt: Option<V>) -> Self
    where
        S: Into<String>,
        V: Into<Val>,
    {
        match value_opt {
            Some(value) => self.insert_display_property(key.into(), value.into()),
            None => self,
        }
    }

    /// Extends the display properties from an iterator of key value pairs.
    pub fn extend_display_properties<I, S, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, V)>,
        S: Into<String>,
        V: Into<Val>
    {
        self.base.display_properties.extend(
            iter.into_iter().map(|(key, value)| (key.into(), value.into()))
        );
        self
    }

    /// Sets the `name` field.
    pub fn set_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>
    {
        self.base.name = Some(name.into());
        self
    }

    /// Removes an existing value from the name field.
    pub fn clear_name(mut self) -> Self {
        self.base.name = None;
        self
    }
}


impl<Id, Val, C, T, DT> CmdCenterPropsBuilder<Id, Val, Option<C>, T, DT> {
    /// Sets the `client` associated with these properties.
    pub fn set_client<S>(self, client: S) -> CmdCenterPropsBuilder<Id, Val, C, T, DT>
    where
        S: Into<C>
    {
        CmdCenterPropsBuilder {
            client: client.into(),
            base: self.base,
            data_type: self.data_type,
        }
    }
}


impl<Id, Val, C, T, DT> CmdCenterPropsBuilder<Id, Val, C, Option<T>, DT> {
    /// Sets the timestamp associated with these properties.
    pub fn set_timestamp(self, timestamp: T) -> CmdCenterPropsBuilder<Id, Val, C, T, DT> {
        CmdCenterPropsBuilder {
            client: self.client,
            base: BasePropertiesBuilder {
                id: self.base.id,
                name: self.base.name,
                epoch: timestamp,
                display_properties: self.base.display_properties,
                props: self.base.props,
            },
            data_type: self.data_type,
        }
    }
}


impl<Id, Val, C, T, DT> CmdCenterPropsBuilder<Id, Val, C, T, Option<DT>> {
    /// Sets the `data_type` this set of properties describes.
    pub fn set_data_type(self, data_type: DT) -> CmdCenterPropsBuilder<Id, Val, C, T, DT> {
        CmdCenterPropsBuilder {
            client: self.client,
            base: self.base,
            data_type,
        }
    }
}


impl<Val, C, T, DT> CmdCenterPropsBuilder<Uuid, Val, C, T, DT>
where
    T: Into<Option<Timestamp>>,
    C: Into<Option<String>>,
    DT: Into<DataType>
{
    /// Builds the [`CmdCenterProps`].
    pub fn build(self) -> CmdCenterProps<Val> {
        self.into()
    }
}



impl<Val, DT> CmdCenterPropsBuilder<Uuid, Val, String, Timestamp, DT> {
    /// Builds the [`SightingProps`].
    pub fn build_sighting_props(self) -> SightingProps<Val> {
        self.into()
    }
}



impl<Id, Val, C, T, DT> CmdCenterPropsBuilder<Option<Id>, Val, C, T, DT> {
    /// Sets the Id.
    pub fn set_id(self, id: Id) -> CmdCenterPropsBuilder<Id, Val, C, T, DT> {
        CmdCenterPropsBuilder {
            base: self.base.set_id(id),
            client: self.client,
            data_type: self.data_type,
        }
    }
}


impl<Id, Val, C, T, DT> CmdCenterPropsBuilder<Id, Val, C, T, DT> {
    /// Overwrites the currently set Id.
    pub fn overwrite_id(mut self, new_id: Id) -> Self {
        self.base.id = new_id;
        self
    }
}
*/
