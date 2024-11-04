//! In progress - specialized Json deserialization with better errors.

/*
use std::borrow::Cow;

use std::collections::BTreeMap;

use serde::de::{self, Deserializer};

use super::{DeserializerImpl, path::{Path, SegTrack}, wrapper::{Delegate, PathContainer}};

// Alias to get rid of a lot of the known generics.
type JsonDeserializerImpl<'t, 'p, 'de, R>
    = DeserializerImpl<'t, 'p, 'de, &'de mut serde_json::Deserializer<R>, BTreeMap<Path, String>>;



pub struct JsonDeserializer<'t, R> {
    inner_de: serde_json::Deserializer<R>,
    seg_track: SegTrack<'t>,
    error_paths: BTreeMap<Path, String>,
}

impl<'t, R> JsonDeserializer<'t, R> {
    fn wrap_inner_de(&mut self) -> JsonDeserializerImpl<'_, '_, '_, R> {
        DeserializerImpl::new(
            &mut self.inner_de,
            Cow::Borrowed(&self.seg_track),
            &mut self.error_paths,
        )
    }
}


impl<'t, R> From<serde_json::Deserializer<R>> for JsonDeserializer<'t, R> {
    fn from(inner_de: serde_json::Deserializer<R>) -> Self {
        Self { inner_de, seg_track: SegTrack::Root, error_paths: BTreeMap::new() }
    }
}

impl<'a, 'de, 't, R> de::Deserializer<'de> for &'a mut JsonDeserializer<'t, R>
where
    R: serde_json::de::Read<'de>
{
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
    {
        self.wrap_inner_de().deserialize_any(visitor)
    }

}


pub struct JsonDelegate<'t, 'p, 'de, A, K, P, R> {
    inner: Delegate<'t, 'p, A, K, P>,
    inner_de: &'de mut serde_json::Deserializer<R>,
}



macro_rules! impl_json_wrapper_visitor_fns {
    ($($fn_name:ident($arg_type:ty)),* $(,)?) => {
        $(
            fn $fn_name<E>(self, v: $arg_type) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                let Self {
                    inner: Delegate { seg_track, error_path, inner_access, .. },
                    inner_de
                } = self;

                inner_access.$fn_name(v)
                    .map_err(|err: E| error_path.insert_path(seg_track, err))
            }
        )*
    };
}


impl<'t, 'p, 'de, A, K, P, R> de::Visitor<'de> for JsonDelegate<'t, 'p, 'de, A, K, P, R>
where
    A: de::Visitor<'de>,
    P: PathContainer,
    R: serde_json::de::Read<'de>,
{
    type Value = A::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.inner_access.expecting(formatter)
    }

    impl_json_wrapper_visitor_fns! {
        visit_bool(bool),
        visit_borrowed_bytes(&'de [u8]),
        visit_borrowed_str(&'de str),
        visit_byte_buf(Vec<u8>),
        visit_char(char),
        visit_f32(f32),
        visit_f64(f64),
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_str(&str),
        visit_string(String),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
    }


    fn visit_enum<E>(self, data: E) -> Result<Self::Value, E::Error>
    where
        E: de::EnumAccess<'de>,
    {
        let Self {
            inner_de,
            inner: Delegate { seg_track, error_path, inner_access, .. },
        } = self;

        inner_access.visit_enum(JsonDelegate {
            inner: Delegate { seg_track, error_path, key: (), inner_access: data },
            inner_de,
        })
    }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let Self {
                inner_de,
                inner: Delegate { seg_track, error_path, inner_access, .. },
            } = self;

            inner_access.visit_map(JsonDelegate {
                inner: Delegate { seg_track, error_path, key: None, inner_access: map },
                inner_de,
            })
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let wrapped = DeserializerImpl::new(deserializer, self.seg_track, self.error_path);

            self.inner_access.visit_newtype_struct(wrapped)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.inner_access.visit_none()
                .map_err(|err| self.error_path.insert_path(self.seg_track, err))
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            self.inner_access.visit_seq(Delegate {
                inner_access: seq,
                seg_track: self.seg_track,
                error_path: self.error_path,
                key: 0usize,
            })
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let wrapped = DeserializerImpl::new(deserializer, self.seg_track, self.error_path);

            self.inner_access.visit_newtype_struct(wrapped)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.inner_access.visit_unit()
                .map_err(|err| self.error_path.insert_path(self.seg_track, err))
        }
}
*/
