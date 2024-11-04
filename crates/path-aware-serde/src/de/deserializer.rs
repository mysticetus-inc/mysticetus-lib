use serde::de;

use super::delegate::Delegate;
use crate::path::{ErrorPath, Track};

pub(crate) struct DeserializerImpl<'t, 'e, D> {
    inner_de: D,
    track: &'t Track<'t>,
    error_path: &'e ErrorPath,
    key_modifier: fn(&mut String),
}

impl<'t, 'e, D> DeserializerImpl<'t, 'e, D> {
    pub(super) fn new(
        inner_de: D,
        track: &'t Track<'t>,
        error_path: &'e ErrorPath,
        key_modifier: fn(&mut String),
    ) -> Self {
        Self {
            inner_de,
            track,
            error_path,
            key_modifier,
        }
    }
}

macro_rules! impl_deserializer_fns {
    ($($fn_name:ident($($arg:ident: $arg_ty:ty),* $(,)?)),* $(,)?) => {
        $(
            fn $fn_name<V>(self, $($arg: $arg_ty,)* visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                self.inner_de.$fn_name($($arg ,)* Delegate {
                    inner_access: visitor,
                    track: self.track,
                    error_path: self.error_path,
                    key_modifier: self.key_modifier,
                    key: (),
                })
            }
        )*
    };
}

impl<'t, 'e, 'de, D> de::Deserializer<'de> for DeserializerImpl<'t, 'e, D>
where
    D: de::Deserializer<'de>,
{
    type Error = D::Error;

    impl_deserializer_fns! {
        deserialize_any(),
        deserialize_bool(),
        deserialize_char(),
        deserialize_str(),
        deserialize_string(),
        deserialize_bytes(),
        deserialize_byte_buf(),
        deserialize_option(),
        deserialize_unit(),
        deserialize_map(),
        deserialize_seq(),
        deserialize_identifier(),
        deserialize_ignored_any(),
        deserialize_i8(),
        deserialize_i16(),
        deserialize_i32(),
        deserialize_i64(),
        deserialize_u8(),
        deserialize_u16(),
        deserialize_u32(),
        deserialize_u64(),
        deserialize_f32(),
        deserialize_f64(),
        deserialize_unit_struct(name: &'static str),
        deserialize_newtype_struct(name: &'static str),
        deserialize_tuple(len: usize),
        deserialize_tuple_struct(name: &'static str, len: usize),
        deserialize_struct(name: &'static str, fields: &'static [&'static str]),
        deserialize_enum(name: &'static str, variants: &'static [&'static str]),
    }
}
