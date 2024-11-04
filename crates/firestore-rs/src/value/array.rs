use std::fmt;

use super::value_ref::ValueRef;

#[derive(Clone, PartialEq)]
pub struct Array {
    values: Vec<protos::firestore::Value>,
}

impl From<Vec<protos::firestore::Value>> for Array {
    fn from(values: Vec<protos::firestore::Value>) -> Self {
        Self { values }
    }
}

impl serde::Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Array {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ArrayVisitor;

        impl<'de> serde::de::Visitor<'de> for ArrayVisitor {
            type Value = Array;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of firestore values")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                super::de::visit_seq_inner(seq).map(Array::from)
            }
        }

        deserializer.deserialize_seq(ArrayVisitor)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ArrayRef<'a> {
    values: &'a [protos::firestore::Value],
}

impl serde::Serialize for ArrayRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.values.len()))?;

        for elem in self.iter() {
            seq.serialize_element(&elem)?;
        }

        seq.end()
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl fmt::Debug for ArrayRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Array {
    pub(super) fn from_proto_value(array_value: protos::firestore::ArrayValue) -> Self {
        Self {
            values: array_value.values,
        }
    }

    pub fn as_ref(&self) -> ArrayRef<'_> {
        ArrayRef {
            values: &self.values,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        self.as_ref().iter()
    }

    pub(super) fn into_proto_value(self) -> protos::firestore::Value {
        protos::firestore::Value {
            value_type: Some(protos::firestore::value::ValueType::ArrayValue(
                protos::firestore::ArrayValue {
                    values: self.values,
                },
            )),
        }
    }

    #[cfg(test)]
    pub fn rand<R: rand::Rng>(rng: &mut R, avail_nesting: usize, allow_nested: bool) -> Self {
        let len = rng.gen_range(0..16_usize);
        let mut dst = Vec::with_capacity(len);

        for _ in 0..len {
            let val = super::Value::rand(rng, avail_nesting, allow_nested).into_proto_value();
            dst.push(val);
        }

        Self::from(dst)
    }
}

impl<'a> ArrayRef<'a> {
    pub(super) fn from_values(values: &'a [protos::firestore::Value]) -> Self {
        Self { values }
    }

    pub fn iter(&self) -> Iter<'a> {
        Iter {
            values: self.values.iter(),
        }
    }
}

pub struct Iter<'a> {
    values: std::slice::Iter<'a, protos::firestore::Value>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ValueRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next().map(ValueRef::from_proto_ref)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.values.len();

        (len, Some(len))
    }
}

impl ExactSizeIterator for Iter<'_> {}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.values.next_back().map(ValueRef::from_proto_ref)
    }
}
