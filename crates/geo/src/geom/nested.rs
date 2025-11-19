//! Coordinate types that contain nested [`Point`]s ([`LineString`]s, [`Polygon`]s, etc.)
use std::fmt;

use serde::{Deserialize, Serialize, de};

use crate::Point;
use crate::util::IndexVisitor;

macro_rules! impl_nested_geometries {
    ($(
        ($name:ident, $inner:ty)
    ),* $(,)?) => {
        $(
            #[doc = "Defines GeoJson "]
            #[doc = stringify!($name)]
            #[doc = " geometry. Thin wrapper around `[`Vec`]<[`"]
            #[doc = stringify!($inner)]
            #[doc = "`]>`."]
            #[repr(transparent)]
            #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
            pub struct $name(Vec<$inner>);

            impl $name {
                #[doc = "Creates an empty [`"]
                #[doc = stringify!($name)]
                #[doc = "`]"]
                #[inline]
                pub fn new() -> Self {
                    Self(Vec::new())
                }

                #[doc = "Creates a [`"]
                #[doc = stringify!($name)]
                #[doc = "`] with 'capacity' pre-allocated"]
                #[inline]
                pub fn with_capacity(capacity: usize) -> Self {
                    Self(Vec::with_capacity(capacity))
                }

                #[doc = "Pushes an [`"]
                #[doc = stringify!($inner)]
                #[doc = "`] to the end of any existing ones."]
                #[inline]
                pub fn push(&mut self, inner: $inner) {
                    self.0.push(inner);
                }

                #[doc = "Returns the number of inner [`"]
                #[doc = stringify!($inner)]
                #[doc = "`] contained within this instance"]
                #[inline]
                pub fn len(&self) -> usize {
                    self.0.len()
                }

                #[doc = "Whether or not this contains any [`"]
                #[doc = stringify!($inner)]
                #[doc = "`]"]
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }

                #[doc = "Returns an [`Iterator`] over references to the inner [`"]
                #[doc = stringify!($inner)]
                #[doc = "`]"]
                #[inline]
                pub fn iter(&self) -> std::slice::Iter<'_, $inner> {
                    self.0.as_slice().iter()
                }

                #[doc = "Returns a slice of the inner [`"]
                #[doc = stringify!($inner)]
                #[doc = "`]"]
                #[inline]
                pub fn as_slice(&self) -> &[$inner] {
                    self.0.as_slice()
                }

                /// Deserialize this type, expecting it to deserialize as nested arrays.
                pub fn deserialize_from_array<'de, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>
                {
                    deserializer.deserialize_seq(NestedVisitor::new(stringify!($name))).map(Self)
                }

                /// Deserialize, expecting it to be deserialized as a map.
                pub fn deserialize_from_map<'de, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>
                {
                    deserializer.deserialize_map(NestedVisitor::new(stringify!($name))).map(Self)
                }
            }

            impl Default for $name {
                #[inline]
                fn default() -> Self {
                    Self::new()
                }
            }

            impl From<$inner> for $name {
                #[inline]
                fn from(inner: $inner) -> Self {
                    Self(vec![inner])
                }
            }

            impl From<Vec<$inner>> for $name {
                #[inline]
                fn from(vec: Vec<$inner>) -> Self {
                    Self(vec)
                }
            }

            impl IntoIterator for $name {
                type Item = $inner;
                type IntoIter = std::vec::IntoIter<$inner>;

                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_iter()
                }
            }

            impl<'a> IntoIterator for &'a $name {
                type Item = &'a $inner;
                type IntoIter = std::slice::Iter<'a, $inner>;

                fn into_iter(self) -> Self::IntoIter {
                    self.0.iter()
                }
            }

            impl Extend<$inner> for $name {
                fn extend<I>(&mut self, iter: I)
                where
                    I: IntoIterator<Item = $inner>
                {
                    self.0.extend(iter)
                }
            }

            impl FromIterator<$inner> for $name {
                fn from_iter<I>(iter: I) -> Self
                where
                    I: IntoIterator<Item = $inner>
                {
                    Self(iter.into_iter().collect::<Vec<$inner>>())
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>
                {
                    deserializer.deserialize_any(NestedVisitor::new(stringify!($name))).map(Self)
                }
            }

            #[cfg(any(test, feature = "random-geom"))]
            impl rand::distr::Distribution<$name> for rand::distr::StandardUniform {
                fn sample<R>(&self, rng: &mut R) -> $name
                where
                    R: rand::Rng + ?Sized
                {
                    // 3-100 range, since we need 3 points minimum for polygons.
                    $name::random_with_len_inner(rng.random_range(3..100), rng)
                }
            }

            #[cfg(any(test, feature = "random-geom"))]
            impl $name {
                /// Testing helpers for generating random coordinates
                pub fn random() -> Self {
                    rand::random()
                }

                pub(crate) fn random_with_len_inner<R>(len: usize, rng: &mut R) -> Self
                where
                    R: rand::Rng + ?Sized
                {
                    let mut container = Vec::with_capacity(len);

                    container.extend(std::iter::repeat_with(|| rng.random::<$inner>()).take(len));

                    $name(container)
                }

                pub fn random_with_len(len: usize) -> Self {
                    let mut thread_rng = rand::rng();
                    Self::random_with_len_inner(len, &mut thread_rng)
                }
            }
        )*
    };
}

impl_nested_geometries! {
    (Line, Point),
    (Polygon, Line),
}

// the impl_nested_geometries macro misses a case of FromIterator for polygons:
impl FromIterator<Vec<Point>> for Polygon {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Vec<Point>>,
    {
        iter.into_iter().map(Line).collect::<Polygon>()
    }
}

struct NestedVisitor<'a, T> {
    expecting: &'a str,
    _inner: std::marker::PhantomData<T>,
}

impl<'a, T> NestedVisitor<'a, T> {
    fn new(expecting: &'a str) -> Self {
        Self {
            expecting,
            _inner: std::marker::PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for NestedVisitor<'_, T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "valid {} coordinates", self.expecting)
    }

    fn visit_seq<S>(self, mut seq_access: S) -> Result<Self::Value, S::Error>
    where
        S: de::SeqAccess<'de>,
    {
        let mut coords = seq_access
            .size_hint()
            .map(Vec::with_capacity)
            .unwrap_or_default();

        while let Some(elem) = seq_access.next_element()? {
            coords.push(elem);
        }

        Ok(coords)
    }

    fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        // since we cant assume the map entries will come in order, we need to store both the
        // index keys and inner values. Once we have them all, we can sort by the index and
        // insert the inner values in order in a new vector
        let mut indexed_coords: Vec<(usize, T)> = map_access
            .size_hint()
            .map(Vec::with_capacity)
            .unwrap_or_default();

        while let Some(index) = map_access.next_key_seed(IndexVisitor)? {
            let value = map_access.next_value()?;
            indexed_coords.push((index, value));
        }

        indexed_coords.sort_by_key(|(idx, _)| *idx);

        // instead of using [`.collect`], allocate a new vector with the exact capacity needed.
        let mut coords = Vec::with_capacity(indexed_coords.len());

        coords.extend(indexed_coords.into_iter().map(|(_, inner)| inner));

        Ok(coords)
    }
}
