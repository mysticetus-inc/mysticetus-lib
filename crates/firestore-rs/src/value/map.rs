use std::collections::HashMap;
use std::fmt;

use super::ValueRef;

#[derive(Clone, PartialEq)]
pub struct Map {
    fields: HashMap<String, protos::firestore::Value>,
}

impl From<HashMap<String, protos::firestore::Value>> for Map {
    fn from(fields: HashMap<String, protos::firestore::Value>) -> Self {
        Self { fields }
    }
}

impl<'de> serde::Deserialize<'de> for Map {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MapVisitor;

        impl<'de> serde::de::Visitor<'de> for MapVisitor {
            type Value = Map;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of firestore values")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                super::de::visit_map_inner(map).map(Map::from)
            }
        }

        deserializer.deserialize_map(MapVisitor)
    }
}

impl serde::Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct MapRef<'a> {
    fields: &'a HashMap<String, protos::firestore::Value>,
}

impl serde::Serialize for MapRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.fields.len()))?;

        for (k, v) in self.iter() {
            map.serialize_entry(&k, &v)?;
        }

        map.end()
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl fmt::Debug for MapRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl Map {
    pub(super) fn from_proto_value(map_value: protos::firestore::MapValue) -> Self {
        Self {
            fields: map_value.fields,
        }
    }

    pub(super) fn into_proto_value(self) -> protos::firestore::Value {
        protos::firestore::Value {
            value_type: Some(protos::firestore::value::ValueType::MapValue(
                protos::firestore::MapValue {
                    fields: self.fields,
                },
            )),
        }
    }

    pub fn as_ref(&self) -> MapRef<'_> {
        MapRef {
            fields: &self.fields,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            fields: self.fields.iter(),
        }
    }

    #[cfg(test)]
    pub fn rand<R: rand::Rng>(rng: &mut R, avail_nesting: usize, allow_nested: bool) -> Self {
        let len = rng.gen_range(0..16_usize);
        let mut dst = HashMap::with_capacity(len);

        for _ in 0..len {
            let key = super::gen_string(rng);
            let val = super::Value::rand(rng, avail_nesting, allow_nested).into_proto_value();
            dst.insert(key, val);
        }

        Self::from(dst)
    }
}

impl<'a> MapRef<'a> {
    pub(super) fn from_fields(fields: &'a HashMap<String, protos::firestore::Value>) -> Self {
        Self { fields }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            fields: self.fields.iter(),
        }
    }
}

pub struct Iter<'a> {
    fields: std::collections::hash_map::Iter<'a, String, protos::firestore::Value>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, ValueRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.fields.next()?;
        Some((k, ValueRef::from_proto_ref(v)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.fields.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for Iter<'_> {}
