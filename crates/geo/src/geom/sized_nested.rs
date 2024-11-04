use std::fmt;

use serde::de;
use serde::ser::{self, SerializeSeq};

use crate::Point;
use crate::util::IndexVisitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SizedLineString<const N: usize> {
    points: [Point; N],
}

impl<const N: usize> ser::Serialize for SizedLineString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(N))?;

        for point in self.points.iter() {
            seq.serialize_element(point)?;
        }

        seq.end()
    }
}

impl<'de, const N: usize> de::Deserialize<'de> for SizedLineString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(SizedLineStringVisitor)
    }
}

struct SizedLineStringVisitor<const N: usize>;

impl<'de, const N: usize> de::Visitor<'de> for SizedLineStringVisitor<N> {
    type Value = SizedLineString<N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a line string with exactly {} points", N)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let points = std::array::try_from_fn(|idx| {
            seq.next_element::<Point>()?.ok_or_else(|| {
                de::Error::custom(format!("only found {} points, expected {}", idx, N))
            })
        })?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom(format!(
                "only expected {} points, found more",
                N
            )));
        }

        Ok(SizedLineString { points })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut indexed_points: [(usize, Point); N] = std::array::try_from_fn(|idx| {
            let index = map.next_key_seed(IndexVisitor)?.ok_or_else(|| {
                de::Error::custom(format!("only found {} points, expected {}", idx, N))
            })?;

            let point = map.next_value::<Point>()?;

            Ok((index, point))
        })?;

        if map
            .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
            .is_some()
        {
            return Err(de::Error::custom(format!(
                "only expected {} points, found more",
                N
            )));
        }

        // verify the points are sorted by index
        indexed_points.sort_by_key(|(index, _)| *index);

        // then convert into just an array of points
        Ok(SizedLineString {
            points: indexed_points.map(|(_, pt)| pt),
        })
    }
}
