//! Path-aware utility functions for [`serde_json`].

use crate::Error;

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

/// Deserializes `O` from a [`&str`].
#[inline]
pub fn deserialize_str<'de, O>(s: &'de str) -> Result<O, Error<serde_json::Error>>
where
    O: serde::Deserialize<'de>,
{
    deserialize(serde_json::de::StrRead::new(s))
}

/// Deserializes `O` from a [`&str`].
#[inline]
pub fn deserialize_slice<'de, O>(s: &'de [u8]) -> Result<O, Error<serde_json::Error>>
where
    O: serde::Deserialize<'de>,
{
    deserialize(serde_json::de::SliceRead::new(s))
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
