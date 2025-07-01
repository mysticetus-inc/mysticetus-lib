use serde::de;
use serde_helpers::seeded_key_capture::SeededKeyCapture;

use super::deserializer::DeserializerImpl;
use crate::path::{ErrorPath, Track};

pub struct Delegate<'t, 'e, A, K> {
    pub(super) track: &'t Track<'t>,
    pub(super) error_path: &'e ErrorPath,
    pub(super) inner_access: A,
    pub(super) key: K,
    pub(super) key_modifier: fn(&mut String),
}

impl<'t, 'e, A, K> Delegate<'t, 'e, A, K> {
    #[inline]
    pub(crate) fn replace_access_and_key<A2, K2>(
        self,
        new_inner: A2,
        new_key: K2,
    ) -> (A, Delegate<'t, 'e, A2, K2>) {
        let wrapper = Delegate {
            track: self.track,
            error_path: self.error_path,
            key_modifier: self.key_modifier,
            inner_access: new_inner,
            key: new_key,
        };

        (self.inner_access, wrapper)
    }

    pub(crate) fn build_deserializer<D>(self, deser: D) -> (A, DeserializerImpl<'t, 'e, D>) {
        let de = DeserializerImpl::new(deser, self.track, self.error_path, self.key_modifier);
        (self.inner_access, de)
    }
}

macro_rules! impl_wrapper_visitor_fns {
    ($($fn_name:ident($arg_type:ty)),* $(,)?) => {
        $(
            fn $fn_name<E>(self, v: $arg_type) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                let Self { track, error_path, inner_access, .. } = self;

                inner_access.$fn_name(v).map_err(move |err: E| {
                    error_path.set(&track);
                    err
                })
            }
        )*
    };
}

impl<'t, 'e, 'de, A, K> de::Visitor<'de> for Delegate<'t, 'e, A, K>
where
    A: de::Visitor<'de>,
{
    type Value = A::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner_access.expecting(formatter)
    }

    impl_wrapper_visitor_fns! {
        visit_bool(bool),
        visit_bytes(&[u8]),
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
            track,
            error_path,
            inner_access,
            key_modifier,
            ..
        } = self;

        inner_access.visit_enum(Delegate {
            track,
            error_path,
            key_modifier,
            key: (),
            inner_access: data,
        })
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let (access, wrapper) = self.replace_access_and_key(map, None);
        access.visit_map(wrapper)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let (access, de) = self.build_deserializer(deserializer);
        access.visit_newtype_struct(de)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.inner_access.visit_none().map_err(|err| {
            self.error_path.set(self.track);
            err
        })
    }

    fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
    where
        S: de::SeqAccess<'de>,
    {
        let (access, delegate) = self.replace_access_and_key(seq, 0usize);
        access.visit_seq(delegate)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let (access, delegate) = self.build_deserializer(deserializer);
        access.visit_some(delegate)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.inner_access.visit_unit().map_err(|err| {
            self.error_path.set(self.track);
            err
        })
    }
}

impl<'t, 'e, 'de, A, K> de::DeserializeSeed<'de> for Delegate<'t, 'e, A, K>
where
    A: de::DeserializeSeed<'de>,
{
    type Value = A::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let (access, delegate) = self.build_deserializer(deserializer);
        access.deserialize(delegate)
    }
}

impl<'t, 'e, 'de, A> de::MapAccess<'de> for Delegate<'t, 'e, A, Option<String>>
where
    A: de::MapAccess<'de>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.inner_access
            .next_key_seed(SeededKeyCapture::new(seed, &mut self.key))
            .map_err(|err| {
                let err_track = match self.key.take() {
                    Some(key) => self.track.add_map_child(key),
                    None => self.track.add_unknown_child(),
                };

                self.error_path.set(&err_track);
                err
            })
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let child_seg_track = match self.key.take() {
            Some(mut key) => {
                (self.key_modifier)(&mut key);
                self.track.add_map_child(key)
            }
            None => self.track.add_unknown_child(),
        };

        let wrapped_seed = Delegate {
            track: &child_seg_track,
            error_path: self.error_path,
            key_modifier: self.key_modifier,
            inner_access: seed,
            key: (),
        };

        self.inner_access
            .next_value_seed(wrapped_seed)
            .map_err(|err| {
                self.error_path.set(&child_seg_track);
                err
            })
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner_access.size_hint()
    }
}

impl<'t, 'e, 'de, A> de::SeqAccess<'de> for Delegate<'t, 'e, A, usize>
where
    A: de::SeqAccess<'de>,
{
    type Error = A::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let nested_child = self.track.add_seq_child(self.key);

        let nested_wrapper = Delegate {
            track: &nested_child,
            inner_access: seed,
            key_modifier: self.key_modifier,
            error_path: self.error_path,
            key: self.key,
        };

        match self.inner_access.next_element_seed(nested_wrapper) {
            Ok(element) => {
                self.key += 1;
                Ok(element)
            }
            Err(error) => {
                self.error_path.set(&nested_child);
                Err(error)
            }
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner_access.size_hint()
    }
}

impl<'t, 'e, 'de, A, K> de::EnumAccess<'de> for Delegate<'t, 'e, A, K>
where
    A: de::EnumAccess<'de>,
{
    type Error = A::Error;
    type Variant = Delegate<'t, 'e, A::Variant, K>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let Self {
            track,
            error_path,
            key,
            inner_access,
            key_modifier,
            ..
        } = self;

        let seed_wrapper = Delegate {
            inner_access: seed,
            error_path,
            key_modifier,
            track,
            key: &key,
        };

        let (value, variant) = inner_access.variant_seed(seed_wrapper)?;

        let variant_wrapper = Delegate {
            inner_access: variant,
            track,
            key_modifier,
            error_path,
            key,
        };

        Ok((value, variant_wrapper))
    }
}

impl<'t, 'e, 'de, A, K> de::VariantAccess<'de> for Delegate<'t, 'e, A, K>
where
    A: de::VariantAccess<'de>,
{
    type Error = A::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.inner_access.unit_variant().map_err(|err| {
            self.error_path.set(self.track);
            err
        })
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (access, delegate) = self.replace_access_and_key(visitor, 0usize);
        access.tuple_variant(len, delegate)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (access, delegate) = self.replace_access_and_key(visitor, Option::<String>::None);
        access.struct_variant(fields, delegate)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let (access, delegate) = self.replace_access_and_key(seed, Option::<String>::None);
        access.newtype_variant_seed(delegate)
    }
}
