//! Path-aware utility functions for [`serde_json`].

use crate::{DeserializerExt, Error};

/// Deserializes the json from a [`serde_json::de::Read`] type. Called internally by
/// [`deserialize_str`], [`deserialize_slice`] and [`deserialize_reader`], but public
/// in case other code needs to be this generic.
pub fn deserialize<'de, I, O>(reader: I) -> Result<O, Error<serde_json::Error>>
where
    O: serde::Deserialize<'de>,
    I: serde_json::de::Read<'de>,
{
    let mut json_de = serde_json::Deserializer::new(reader);

    let output = <O as super::DeserializeExt<'de>>::deserialize_path_aware(&mut json_de)?;

    json_de.end().map_err(|err| Error::new(err, None))?;

    Ok(output)
}

/// Deserializes the json from a [`serde_json::de::Read`] type, with a seed.
pub fn deserialize_seed<'de, S, I>(seed: S, reader: I) -> Result<S::Value, Error<serde_json::Error>>
where
    S: serde::de::DeserializeSeed<'de>,
    I: serde_json::de::Read<'de>,
{
    let mut json_de = serde_json::Deserializer::new(reader);

    let output = seed.deserialize(json_de.make_path_aware())?;

    json_de.end().map_err(|err| Error::new(err, None))?;

    Ok(output)
}

/// Deserializes `O` from a [`&str`].
#[inline]
pub fn deserialize_str<'de, O>(s: &'de str) -> Result<O, Error<serde_json::Error>>
where
    O: serde::Deserialize<'de>,
{
    deserialize(serde_json::de::StrRead::new(s))
}

/// Deserializes from a [`&str`] and a seed.
#[inline]
pub fn deserialize_str_seed<'de, S>(
    seed: S,
    s: &'de str,
) -> Result<S::Value, Error<serde_json::Error>>
where
    S: serde::de::DeserializeSeed<'de>,
{
    deserialize_seed(seed, serde_json::de::StrRead::new(s))
}

/// Deserializes `O` from a [`&[u8]`].
#[inline]
pub fn deserialize_slice<'de, O>(s: &'de [u8]) -> Result<O, Error<serde_json::Error>>
where
    O: serde::Deserialize<'de>,
{
    deserialize(serde_json::de::SliceRead::new(s))
}

/// Deserializes from a [`&[u8]`] and a seed.
#[inline]
pub fn deserialize_slice_seed<'de, S>(
    seed: S,
    s: &'de [u8],
) -> Result<S::Value, Error<serde_json::Error>>
where
    S: serde::de::DeserializeSeed<'de>,
{
    deserialize_seed(seed, serde_json::de::SliceRead::new(s))
}

/// Deserializes `O` from a `R`, where <code>R: [std::io::Read]</code>.
#[inline]
pub fn deserialize_reader<'de, O, R>(reader: R) -> Result<O, Error<serde_json::Error>>
where
    O: serde::de::Deserialize<'de>,
    R: std::io::Read,
{
    deserialize(serde_json::de::IoRead::new(reader))
}

/// Deserializes from a reader `R` and a seed, where <code>R: [std::io::Read]</code>.
#[inline]
pub fn deserialize_reader_seed<'de, S, R>(
    seed: S,
    reader: R,
) -> Result<S::Value, Error<serde_json::Error>>
where
    S: serde::de::DeserializeSeed<'de>,
    R: std::io::Read,
{
    deserialize_seed(seed, serde_json::de::IoRead::new(reader))
}
