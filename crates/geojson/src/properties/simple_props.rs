//! [`SimpleProperty`], a single key/value [`PropertyMap`] implementor.

use serde::{Deserialize, Serialize};

use super::PropertyMap;

/// A simple [`PropertyMap`] implementing type, containing a single key/value pair.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SimpleProperty<T>(Option<(String, T)>);

/// An iterator over a reference to the single value in [`SimpleProperty`].
pub struct SinglePropIter<'a, T> {
    inner: Option<&'a Option<(String, T)>>,
}

impl<'a, T> Iterator for SinglePropIter<'a, T> {
    type Item = (&'a String, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()?.as_ref().map(|(key, val)| (key, val))
    }
}

impl<'a, T> ExactSizeIterator for SinglePropIter<'a, T> {
    fn len(&self) -> usize {
        match self.inner {
            Some(Some(_)) => 1,
            _ => 0,
        }
    }
}

/// An iterator over the mutable reference to the single value in [`SimpleProperty`].
pub struct SinglePropIterMut<'a, T> {
    inner: Option<&'a mut Option<(String, T)>>,
}

impl<'a, T> Iterator for SinglePropIterMut<'a, T> {
    type Item = (&'a String, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()?.as_mut().map(|(key, val)| (&*key, val))
    }
}

impl<'a, T> ExactSizeIterator for SinglePropIterMut<'a, T> {
    fn len(&self) -> usize {
        match self.inner {
            Some(Some(_)) => 1,
            _ => 0,
        }
    }
}

impl<T> PropertyMap for SimpleProperty<T>
where
    T: Default,
{
    type Value = T;

    type Iter<'a>
        = SinglePropIter<'a, Self::Value>
    where
        T: 'a;

    type IterMut<'a>
        = SinglePropIterMut<'a, Self::Value>
    where
        T: 'a;

    type IntoIter = std::option::IntoIter<(String, Self::Value)>;

    fn insert<S>(&mut self, key: S, value: Self::Value) -> Option<Self::Value>
    where
        S: Into<String>,
    {
        self.0.replace((key.into(), value)).map(|(_, val)| val)
    }

    fn remove<S>(&mut self, key: S) -> Option<(String, Self::Value)>
    where
        S: AsRef<str>,
    {
        match self.0.as_ref() {
            Some((existing, _)) if existing.as_str() == key.as_ref() => self.0.take(),
            _ => None,
        }
    }

    fn get<S>(&self, key: S) -> Option<&Self::Value>
    where
        S: AsRef<str>,
    {
        match self.0.as_ref() {
            Some((existing_key, val)) if key.as_ref() == existing_key.as_str() => Some(val),
            _ => None,
        }
    }

    fn get_mut<S>(&mut self, key: S) -> Option<&mut Self::Value>
    where
        S: AsRef<str>,
    {
        match self.0.as_mut() {
            Some((existing_key, val)) if key.as_ref() == existing_key.as_str() => Some(val),
            _ => None,
        }
    }

    fn get_or_insert<S>(&mut self, key: S, value: Self::Value) -> &mut Self::Value
    where
        S: Into<String>,
    {
        &mut self.0.get_or_insert((key.into(), value)).1
    }

    fn get_or_insert_with<S, F>(&mut self, key: S, func: F) -> &mut Self::Value
    where
        S: Into<String>,
        F: FnOnce() -> Self::Value,
    {
        &mut self.0.get_or_insert_with(|| (key.into(), func())).1
    }

    fn iter(&self) -> Self::Iter<'_> {
        SinglePropIter {
            inner: Some(&self.0),
        }
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        SinglePropIterMut {
            inner: Some(&mut self.0),
        }
    }

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }

    fn len(&self) -> usize {
        match self.0.as_ref() {
            Some(_) => 1,
            None => 0,
        }
    }
}
