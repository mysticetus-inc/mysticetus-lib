use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object<S = String, M = HashMap<String, String>> {
    pub name: S,
    #[serde(with = "serde_mime")]
    pub content_type: mime_guess::Mime,
    #[serde(skip_serializing, with = "serde_crc32c")]
    pub crc32c: u32,
    #[serde(skip_serializing, with = "serde_md5")]
    pub md5_hash: Option<[u8; 16]>,
    #[serde(default)]
    pub metadata: M,
    #[serde(skip_serializing, with = "serde_int")]
    pub generation: i64,
    #[serde(skip_serializing, with = "serde_int")]
    pub size: u64,
    #[serde(skip_serializing)]
    pub time_created: timestamp::Timestamp,
    #[serde(skip_serializing)]
    pub updated: timestamp::Timestamp,
}

impl<S, M> Object<S, M>
where
    S: ToOwned,
    M: ToOwned,
{
    pub fn into_owned(self) -> Object<S::Owned, M::Owned> {
        Object {
            name: self.name.to_owned(),
            content_type: self.content_type,
            generation: self.generation,
            size: self.size,
            crc32c: self.crc32c,
            md5_hash: self.md5_hash,
            metadata: self.metadata.to_owned(),
            time_created: self.time_created,
            updated: self.updated,
        }
    }
}

impl Object {
    pub(super) const FIELDS: &'static str =
        "name,contentType,crc32c,md5Hash,metadata,generation,size,timeCreated,updated";
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewObject<S = String, M = HashMap<String, String>> {
    pub name: S,
    #[serde(with = "serde_mime")]
    pub content_type: mime_guess::Mime,
    #[serde(default)]
    pub metadata: M,
    #[serde(skip_serializing, with = "serde_int")]
    pub size: u64,
}

struct Base64<const SIZE: usize>([u8; SIZE]);

impl<'de, const SIZE: usize> serde::Deserialize<'de> for Base64<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&'de str as serde::Deserialize<'de>>::deserialize(deserializer)?;
        let mut dst = [0; SIZE];

        base64::decode_config_slice(s.as_bytes(), base64::STANDARD, &mut dst)
            .map_err(serde::de::Error::custom)?;

        Ok(Self(dst))
    }
}

#[allow(dead_code)] // in dev
mod serde_int {
    use serde::de;

    pub fn serialize<I, S>(int: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        I: itoa::Integer + Copy,
        S: serde::Serializer,
    {
        let mut buf = itoa::Buffer::new();
        serializer.serialize_str(buf.format(*int))
    }

    pub fn deserialize<'de, D, I>(deserializer: D) -> Result<I, D::Error>
    where
        D: de::Deserializer<'de>,
        I: std::str::FromStr,
        I::Err: std::fmt::Display,
    {
        <&'de str as serde::Deserialize<'de>>::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

#[allow(dead_code)] // in dev
mod serde_crc32c {
    use serde::{Deserialize, de};

    pub fn serialize<S>(int: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = itoa::Buffer::new();
        let int_str = buf.format(*int);

        let disp =
            base64::display::Base64Display::with_config(int_str.as_bytes(), base64::STANDARD);
        serializer.collect_str(&disp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let decoded = super::Base64::<4>::deserialize(deserializer)?;

        Ok(u32::from_be_bytes(decoded.0))
    }
}

#[allow(dead_code)] // in dev
mod serde_md5 {
    use serde::{Deserialize, de};

    pub fn serialize<S>(md5: &Option<[u8; 16]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match md5 {
            Some(bytes) => {
                let disp = base64::display::Base64Display::with_config(bytes, base64::STANDARD);
                serializer.collect_str(&disp)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 16]>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let decoded = Option::<super::Base64<16>>::deserialize(deserializer)?;

        match decoded {
            Some(super::Base64(bytes)) => Ok(Some(bytes)),
            None => Ok(None),
        }
    }
}

mod serde_mime {
    use std::fmt;

    use serde::de;

    pub fn serialize<S>(mime: &mime_guess::Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(mime)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<mime_guess::Mime, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(MimeVisitor)
    }

    struct MimeVisitor;

    impl<'de> de::Visitor<'de> for MimeVisitor {
        type Value = mime_guess::Mime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a standard MIME type")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<mime_guess::Mime>().map_err(de::Error::custom)
        }
    }
}
