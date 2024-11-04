//! [`Properties`]/[`PropertyMap`] traits, and built-in implementors of those traits.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::iter::Map;

use timestamp::Timestamp;

pub mod base_props;
pub mod cmd_center_props;
pub mod simple_props;

/// Generic trait for a map of properties, with string keys, and a implementor defined value type.
///
/// Implemented on both [`BTreeMap`] and [`HashMap`] with [`String`] keys, and generic values.
pub trait PropertyMap: Sized {
    /// The value type contained by this map.
    type Value;

    /// A iterator type that iterates over references to the keys and values.
    type Iter<'a>: Iterator<Item = (&'a String, &'a Self::Value)>
    where
        Self: 'a;

    /// A iterator type that iterates over key references, and mutable value references.
    type IterMut<'a>: Iterator<Item = (&'a String, &'a mut Self::Value)>
    where
        Self: 'a;

    /// A iterator over the owned keys and values.
    type IntoIter: Iterator<Item = (String, Self::Value)>;

    /// Inserts a value into the map, returning the previous value, if it was set.
    fn insert<S>(&mut self, key: S, value: Self::Value) -> Option<Self::Value>
    where
        S: Into<String>;

    /// Removes an entry from the map, if it exists.
    fn remove<S>(&mut self, key: S) -> Option<(String, Self::Value)>
    where
        S: AsRef<str>;

    /// Given a key, get a reference to the value, if it exists.
    fn get<S>(&self, key: S) -> Option<&Self::Value>
    where
        S: AsRef<str>;

    /// Given a key, get a mutable reference to the value, if it exists.
    fn get_mut<S>(&mut self, key: S) -> Option<&mut Self::Value>
    where
        S: AsRef<str>;

    /// If the value at that key exsits, returns a mutable reference to it. If it does not exist,
    /// 'value' is inserted in that spot, and a mutable reference to that is returned. If the
    /// value type is not cheap to build or clone, use [`get_or_insert_with`] instead to only
    /// build the value when an existing value is missing.
    ///
    /// [`get_or_insert_with`]: [`PropertyMap::get_or_insert_with`]
    fn get_or_insert<S>(&mut self, key: S, value: Self::Value) -> &mut Self::Value
    where
        S: Into<String>;

    /// If the value at that key exists, returns a mutable reference to it. If it does not, the
    /// value is filled with the [`Default`] for [`Self::Value`], and a mutable reference to that
    /// is returned.
    fn get_or_insert_default<S>(&mut self, key: S) -> &mut Self::Value
    where
        S: AsRef<str>,
        Self::Value: Default,
    {
        self.get_or_insert(key.as_ref(), Self::Value::default())
    }

    /// If the value at that key exists, returns a mutable reference to it. If it does not, the
    /// value is filled with value provided by the function 'F', and the mutable reference to that
    /// is returned.
    fn get_or_insert_with<S, F>(&mut self, key: S, func: F) -> &mut Self::Value
    where
        S: Into<String>,
        F: FnOnce() -> Self::Value;

    /// Returns the iterator over the references of the keys/values
    fn iter(&self) -> Self::Iter<'_>;

    /// Returns an iterator over the key references, and mutable value references.
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// Moves out of 'self', returning an iterator over the owned keys + values. This is
    /// essentially the same as a [`IntoIterator::into_iter`] call under the hood.
    fn into_iter(self) -> Self::IntoIter;

    /// Checks if there's a value at the given key.
    fn contains<S>(&self, key: S) -> bool
    where
        S: AsRef<str>,
    {
        self.get(key.as_ref()).is_some()
    }

    /// Returns an iterator over references to the keys in the map.
    fn keys(&self) -> KeyMapIter<Self::Iter<'_>, &String, &Self::Value> {
        self.iter().map(KeyExtractor::new())
    }

    /// Returns an iterator over references to the `[`PropertyMap::Value`]s` in the map.
    fn values(&self) -> ValueMapIter<Self::Iter<'_>, &String, &Self::Value> {
        self.iter().map(ValueExtractor::new())
    }

    /// Returns an iterator over mutable references to the `[`PropertyMap::Value`]s`.
    fn values_mut(&mut self) -> ValueMapIter<Self::IterMut<'_>, &String, &mut Self::Value> {
        self.iter_mut().map(ValueExtractor::new())
    }

    /// Returns an iterator over the owned keys in the map.
    fn into_keys(self) -> KeyMapIter<Self::IntoIter, String, Self::Value> {
        self.into_iter().map(KeyExtractor::new())
    }

    /// Returns an iterator over the owned `[`PropertyMap::Value`]s` in the map.
    fn into_values(self) -> ValueMapIter<Self::IntoIter, String, Self::Value> {
        self.into_iter().map(ValueExtractor::new())
    }

    /// The number of entries in this [`PropertyMap`]
    fn len(&self) -> usize;

    /// Whether or not the [`PropertyMap`] is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Helper trait for implementing [`PropertyMap`]. If an inner type already implements
/// [`PropertyMap`], implementing this also implies the full [`PropertyMap`] implementation on
/// [`Self`].
pub trait InnerPropertyMap {
    /// The type of map used under the hood.
    type Map: PropertyMap;

    /// Returns a reference to the underlying map.
    fn property_map(&self) -> &Self::Map;

    /// Returns a mutable reference to the underlying map.
    fn property_map_mut(&mut self) -> &mut Self::Map;

    /// Consumes [`Self`] and returns the owned underlying map.
    fn into_property_map(self) -> Self::Map;
}

impl<T> PropertyMap for T
where
    T: InnerPropertyMap,
{
    type Value = <<T as InnerPropertyMap>::Map as PropertyMap>::Value;

    type Iter<'a>
        = <<T as InnerPropertyMap>::Map as PropertyMap>::Iter<'a>
    where
        Self: 'a;

    type IterMut<'a>
        = <<T as InnerPropertyMap>::Map as PropertyMap>::IterMut<'a>
    where
        Self: 'a;

    type IntoIter = <<T as InnerPropertyMap>::Map as PropertyMap>::IntoIter;

    fn insert<S>(&mut self, key: S, value: Self::Value) -> Option<Self::Value>
    where
        S: Into<String>,
    {
        self.property_map_mut().insert(key.into(), value)
    }

    fn remove<S>(&mut self, key: S) -> Option<(String, Self::Value)>
    where
        S: AsRef<str>,
    {
        self.property_map_mut().remove(key.as_ref())
    }

    fn get_mut<S>(&mut self, key: S) -> Option<&mut Self::Value>
    where
        S: AsRef<str>,
    {
        self.property_map_mut().get_mut(key.as_ref())
    }

    fn get_or_insert<S>(&mut self, key: S, value: Self::Value) -> &mut Self::Value
    where
        S: Into<String>,
    {
        self.property_map_mut().get_or_insert(key.into(), value)
    }

    fn get_or_insert_with<S, F>(&mut self, key: S, func: F) -> &mut Self::Value
    where
        S: Into<String>,
        F: FnOnce() -> Self::Value,
    {
        self.property_map_mut().get_or_insert_with(key.into(), func)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.property_map().iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.property_map_mut().iter_mut()
    }

    fn into_iter(self) -> Self::IntoIter {
        self.into_property_map().into_iter()
    }

    fn get<S>(&self, key: S) -> Option<&Self::Value>
    where
        S: AsRef<str>,
    {
        self.property_map().get(key.as_ref())
    }

    fn len(&self) -> usize {
        self.property_map().len()
    }
}

macro_rules! impl_extractors {
    ($(
        $name:ident<$key:ident, $val:ident>: $output:ident => $tuple_idx:tt
    ),* $(,)?) => {
        $(

            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
            pub struct $name<$key, $val> {
                _marker: std::marker::PhantomData<($key, $val)>
            }

            impl<$key, $val> $name<$key, $val> {
                fn new() -> Self {
                    Self { _marker: std::marker::PhantomData }
                }
            }

            impl<$key, $val> FnOnce<(($key, $val),)> for $name<$key, $val> {
                type Output = $output;

                extern "rust-call" fn call_once(self, args: (($key, $val),)) -> Self::Output {
                    args.0.$tuple_idx
                }
            }


            impl<$key, $val> Fn<(($key, $val),)> for $name<$key, $val> {
                extern "rust-call" fn call(&self, args: (($key, $val),)) -> Self::Output {
                    args.0.$tuple_idx
                }
            }

            impl<$key, $val> FnMut<(($key, $val),)> for $name<$key, $val> {
                extern "rust-call" fn call_mut(&mut self, args: (($key, $val),)) -> Self::Output {
                    args.0.$tuple_idx
                }
            }

        )*
    };
}

impl_extractors! {
    KeyExtractor<K, V>: K => 0,
    ValueExtractor<K, V>: V => 1,
}

// Aliases to shorten the names of 'keys'/'values' return types in [`PropertyMap`]
type KeyMapIter<I, K, V> = Map<I, KeyExtractor<K, V>>;
type ValueMapIter<I, K, V> = Map<I, ValueExtractor<K, V>>;

/// An extension on [`PropertyMap`], this trait defines specific for GeoJson properties,
/// such as setting/getting an Id, timestamp, and a nested display properties map.
pub trait Properties: PropertyMap {
    /// The type of Id stored in this property. Must be [`Copy`], [`Ord`] and [`Hash`]-able, that
    /// way all Id's are cheap to throw around, orderable and usable in a [`HashMap`]/[`HashSet`].
    ///
    /// [`HashSet`]: [`std::collections::HashSet`]
    type Id: Clone + Copy + Ord + Hash;

    /// The underlying, owned name type. By default, `NameRef<'a> = &'a Name`, but this
    /// isn't always the nicest when working with types like [`Option`] or [`Cow`].
    ///
    /// For example, this is useful in a scenario like:
    ///
    /// ```
    /// #![feature(generic_associated_types)]
    /// # use geojson::properties::base_props::BaseProperties;
    /// # use geojson::properties::{InnerPropertyMap, Properties};
    /// # use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct Props {
    ///     // the specific implementation doesn't matter
    /// #   base: BaseProperties<uuid::Uuid, Option<timestamp::Timestamp>, Option<String>, ()>,
    /// }
    ///
    /// # impl InnerPropertyMap for Props {
    /// #   type Map = BaseProperties<uuid::Uuid, Option<timestamp::Timestamp>, Option<String>, ()>;
    /// #   fn property_map(&self) -> &Self::Map { &self.base }
    /// #   fn property_map_mut(&mut self) -> &mut Self::Map { &mut self.base }
    /// #   fn into_property_map(self) -> Self::Map { self.base }
    /// # }
    ///
    /// impl Properties for Props {
    ///     // Using an optional, owned string means `name` would by default return
    ///     // `&Option<String>`, which is far from the most idiomatic way to return that type.
    ///     type Name = Option<String>;
    ///
    ///     // By overriding the default with with a more sensible type, idiomatic code can be
    ///     // maintained.
    ///     type NameRef<'a> = Option<&'a str>
    ///     where
    ///         Self: 'a;
    ///
    ///     fn name(&self) -> Option<&'_ str> {
    ///         // ...
    /// #       self.base.name()
    ///     }
    ///
    ///     fn name_mut(&mut self) -> &mut Option<String> {
    ///         // ...
    /// #       self.base.name_mut()
    ///     }
    ///
    ///     // ... Omitting other associated types + methods
    ///     # type Id = uuid::Uuid;
    ///     # type RequiredArgs = <BaseProperties<()> as Properties>::RequiredArgs;
    ///     # fn new(_: Self::RequiredArgs) -> Self { todo!() }
    ///     # fn id(&self) -> Self::Id { todo!() }
    ///     # fn set_id(&mut self, _: Self::Id) { todo!() }
    /// }
    /// ```
    ///
    /// [`Cow`]: [`std::borrow::Cow`]
    type Name;

    /// The reference to a name, returned by [`name`]. By default, it's set to
    /// [`&'a Properties::Name`].
    ///
    /// [`name`]: [`Properties::name`]
    type NameRef<'a>
        = &'a Self::Name
    where
        Self: 'a;

    /// The arguments required to instantiate a new [`Self`].
    type RequiredArgs;

    /// Instantiates a new set of properties with a given [`Properties::Id`].
    fn new(args: Self::RequiredArgs) -> Self;

    /// Returns the [`Properties::Id`] for this set of properties.
    fn id(&self) -> Self::Id;

    /// Sets the [`Properties::Id`] for this set of properties. Ideally calling
    /// [`Properties::new`] is preferred.
    fn set_id(&mut self, id: Self::Id);

    /// Returns a reference (or cheap [`Copy`]) or the name.
    fn name(&self) -> Self::NameRef<'_>;

    /// Sets the name, returning the previously set name
    fn set_name<I>(&mut self, new_name: I) -> Self::Name
    where
        I: Into<Self::Name>,
    {
        std::mem::replace(self.name_mut(), new_name.into())
    }

    fn name_mut(&mut self) -> &mut Self::Name;
}

/// A trait implemented by [`Properties`] that also contain a separate [`PropertyMap`] for
/// things that an end-user will be shown.
pub trait DisplayProps: Properties {
    type DisplayProps: PropertyMap;

    /// Returns a reference to the underlying [`DisplayProps`].
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    fn display_props(&self) -> &Self::DisplayProps;

    /// Returns a mutable reference to the underlying [`DisplayProps`].
    ///
    /// [`DisplayProps`]: [`DisplayProps::DisplayProps`]
    fn display_props_mut(&mut self) -> &mut Self::DisplayProps;
}

/// Extension of [`Properties`], where the implemnting type contains a required [`Timestamp`].
pub trait TimedProps: Properties {
    /// Returns the timestamp assigned to this set of [`Properties`].
    fn timestamp(&self) -> Timestamp;

    /// Sets the timestamp for this set of [`Properties`].
    fn set_timestamp(&mut self, timestamp: Timestamp);
}

macro_rules! impl_for_std_maps {
    ($($map_name:ident),* $(,)?) => {
        $(
            impl<V> PropertyMap for $map_name<String, V> {
                type Value = V;

                type Iter<'a> = <&'a $map_name<String, V> as IntoIterator>::IntoIter
                where
                    V: 'a;

                type IterMut<'a> = <&'a mut $map_name<String, V> as IntoIterator>::IntoIter
                where
                    V: 'a;

                type IntoIter = <$map_name<String, V> as IntoIterator>::IntoIter;

                fn insert<S>(&mut self, key: S, value: Self::Value) -> Option<Self::Value>
                where
                    S: Into<String>,
                {
                    self.insert(key.into(), value)
                }

                fn remove<S>(&mut self, key: S) -> Option<(String, Self::Value)>
                where
                    S: AsRef<str>
                {
                    self.remove_entry(key.as_ref())
                }

                fn get<S>(&self, key: S) -> Option<&Self::Value>
                where
                    S: AsRef<str>
                {
                    self.get(key.as_ref())
                }

                fn get_mut<S>(&mut self, key: S) -> Option<&mut Self::Value>
                where
                    S: AsRef<str>
                {
                    self.get_mut(key.as_ref())
                }

                fn get_or_insert<S>(&mut self, key: S, value: Self::Value) -> &mut Self::Value
                where
                    S: Into<String>
                {
                    self.entry(key.into()).or_insert(value)
                }

                fn get_or_insert_with<S, F>(&mut self, key: S, func: F) -> &mut Self::Value
                where
                    S: Into<String>,
                    F: FnOnce() -> Self::Value
                {
                    self.entry(key.into()).or_insert_with(func)
                }

                fn iter(&self) -> Self::Iter<'_> {
                    self.iter()
                }

                fn iter_mut(&mut self) -> Self::IterMut<'_> {
                    self.iter_mut()
                }

                fn into_iter(self) -> Self::IntoIter {
                    IntoIterator::into_iter(self)
                }

                fn len(&self) -> usize {
                    self.len()
                }
            }
        )*
    };
}

impl_for_std_maps!(HashMap, BTreeMap);

impl PropertyMap for serde_json::Map<String, serde_json::Value> {
    type Value = serde_json::Value;

    type Iter<'a> = serde_json::map::Iter<'a>;
    type IterMut<'a> = serde_json::map::IterMut<'a>;
    type IntoIter = serde_json::map::IntoIter;

    fn insert<S>(&mut self, key: S, value: Self::Value) -> Option<Self::Value>
    where
        S: Into<String>,
    {
        self.insert(key.into(), value)
    }

    fn remove<S>(&mut self, key: S) -> Option<(String, Self::Value)>
    where
        S: AsRef<str>,
    {
        self.remove_entry(key.as_ref())
    }

    fn get<S>(&self, key: S) -> Option<&Self::Value>
    where
        S: AsRef<str>,
    {
        self.get(key.as_ref())
    }

    fn get_mut<S>(&mut self, key: S) -> Option<&mut Self::Value>
    where
        S: AsRef<str>,
    {
        self.get_mut(key.as_ref())
    }

    fn get_or_insert<S>(&mut self, key: S, value: Self::Value) -> &mut Self::Value
    where
        S: Into<String>,
    {
        self.entry(key.into()).or_insert(value)
    }

    fn get_or_insert_with<S, F>(&mut self, key: S, func: F) -> &mut Self::Value
    where
        S: Into<String>,
        F: FnOnce() -> Self::Value,
    {
        self.entry(key.into()).or_insert_with(func)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.iter_mut()
    }

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self)
    }

    fn len(&self) -> usize {
        self.len()
    }
}
