//! Timestamp serialization methods + impl

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::Timestamp;

// Serialization functions
impl Timestamp {
    /// Serializes this [`Timestamp`], represented as floating point seconds since the unix epoch.
    /// The default implementation of [`Serialize::serialize`] calls this
    /// under the hood.
    pub fn serialize_as_seconds_f64<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.as_seconds_f64())
    }

    /// Serializes this [`Timestamp`] as an integer number of seconds relative to the unix Epoch.
    pub fn serialize_as_seconds_int<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.as_seconds())
    }

    /// Serializes this [`Timestamp`], represented as floating point milliseconds since the unix
    /// epoch.
    pub fn serialize_as_millis_f64<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.as_millis_f64())
    }

    /// Serializes this [`Timestamp`], represented as an integer number of microseconds since the
    /// unix epoch.
    pub fn serialize_as_micros_i64<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.as_micros())
    }

    /// Serializes as a RFC 3339 (a subset of ISO 8601) valid datetime string.
    /// (i.e '1996-12-19T16:39:57Z')
    pub fn serialize_as_iso8601<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_iso8601())
    }

    /// Serializes as a RFC 3339 (a subset of ISO 8601) valid datetime string, separated by a
    /// space. (i.e '1996-12-19 16:39:57Z')
    pub fn serialize_as_space_separated_iso8601<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_space_separated_iso8601())
    }

    /// Serializes as a RFC 2822 date time string.
    /// (i.e 'Tue, 1 Jul 2003 10:52:37 +0200')
    pub fn serialize_as_rfc2822<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_datetime().to_rfc2822())
    }

    /// Serializes as the raw number of nanoseconds since the unix epoch. This format is the same
    /// as the internal representation of a [`Timestamp`], so theoretically this is the most
    /// efficient.
    pub fn serialize_as_nanos<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i128(self.as_nanos())
    }

    /// If [`Some`], behaves like [`Timestamp::serialize_as_iso8601`], otherwise calls
    /// [`Serializer::serialize_none`]
    pub fn serialize_opt_as_iso8601<S>(opt: &Option<Self>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt.as_ref() {
            Some(ts) => serializer.serialize_some(&ts.as_iso8601()),
            None => serializer.serialize_none(),
        }
    }

    /// If [`Some`], behaves like [`Timestamp::serialize_as_seconds_f64`], otherwise calls
    /// [`Serializer::serialize_none`]
    pub fn serialize_as_seconds_opt<S>(opt: &Option<Self>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt.as_ref() {
            Some(ts) => serializer.serialize_some(&ts.as_seconds_f64()),
            None => serializer.serialize_none(),
        }
    }

    /// Serializes as a google/proto encoded timestamp:
    ///
    /// `{ seconds: '...', nanos: '...' }`
    pub fn serialize_as_proto<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(2))?;

        map_ser.serialize_entry("seconds", &self.as_seconds())?;
        map_ser.serialize_entry("nanos", &self.subsec_nanos())?;

        map_ser.end()
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serialize_as_seconds_f64(serializer)
    }
}
