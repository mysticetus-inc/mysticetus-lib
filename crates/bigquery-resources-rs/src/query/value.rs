use serde::de::{self, IntoDeserializer};

use crate::table::{FieldMode, FieldType, TableFieldSchema};

pub struct ValueMapSeed<'a, S> {
    // mode, ty and field_name are all parts from an instance of TableFieldSchema,
    // but with the name already coerced to &str so we dont need another generic
    // parameter (in TableFieldSchema<S>)
    mode: FieldMode,
    ty: FieldType,
    field_name: &'a str,
    seed: S,
}

impl<'a, S> ValueMapSeed<'a, S> {
    pub(super) fn new<S2: AsRef<str>>(field: &'a TableFieldSchema<S2>, seed: S) -> Self {
        Self {
            mode: field.mode,
            ty: field.ty,
            field_name: field.name.as_ref(),
            seed,
        }
    }
}

impl<'de, S> de::DeserializeSeed<'de> for ValueMapSeed<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, S> de::Visitor<'de> for ValueMapSeed<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an object containing {'v': ")?;
        write!(formatter, "{}}}", std::any::type_name::<S::Value>())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        #[derive(serde::Deserialize)]
        enum Field {
            #[serde(rename = "v")]
            V,
            #[serde(other)]
            Other,
        }

        let mut seed = Some(self.seed);
        let mut value = None;

        while let Some(field) = map.next_key()? {
            match field {
                Field::V => match seed.take() {
                    Some(seed) => {
                        let value_seed = ValueSeed {
                            mode: self.mode,
                            ty: self.ty,
                            field_name: self.field_name,
                            seed,
                        };

                        #[cfg(feature = "debug-json")]
                        let value_seed =
                            serde_helpers::debug_visitor::DebugVisitor::stdout(value_seed);

                        value = Some(map.next_value_seed(value_seed)?);
                    }
                    None => return Err(de::Error::duplicate_field("v")),
                },
                Field::Other => _ = map.next_value::<de::IgnoredAny>()?,
            }
        }

        value.ok_or_else(|| de::Error::missing_field("v"))
    }
}

struct ValueSeed<'a, S> {
    mode: FieldMode,
    ty: FieldType,
    field_name: &'a str,
    seed: S,
}

fn deserialize_according_to_type<'de, S, D>(
    seed: ValueSeed<'_, S>,
    deserializer: D,
) -> Result<S::Value, D::Error>
where
    S: de::DeserializeSeed<'de>,
    D: de::Deserializer<'de>,
{
    match seed.ty {
        FieldType::String
        | FieldType::Bytes
        | FieldType::Timestamp
        | FieldType::Time
        | FieldType::Date
        | FieldType::DateTime
        | FieldType::BigNumeric
        | FieldType::Numeric
        | FieldType::Integer
        | FieldType::Float
        | FieldType::Json => deserializer.deserialize_string(seed),
        _ => panic!("unknown format for bigquery encoded {:?}", seed.ty),
    }
}

impl<'de, S> de::DeserializeSeed<'de> for ValueSeed<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.mode {
            FieldMode::Nullable => deserializer.deserialize_option(self),
            FieldMode::Repeated => deserializer.deserialize_seq(self),
            FieldMode::Required => deserialize_according_to_type(self, deserializer),
        }
    }
}

impl<'de, S> de::Visitor<'de> for ValueSeed<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a {:?} {:?} encoded via the BigQuery REST API (field name '{}')",
            self.mode, self.ty, self.field_name
        )
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.seed
            .deserialize(serde::de::value::UnitDeserializer::new())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_unit()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.ty {
            FieldType::String
            | FieldType::Bytes
            | FieldType::BigNumeric
            | FieldType::Numeric
            | FieldType::Json => {
                // TODO: make this less hacky and more efficient
                // (there should be a way to wrap this visitor
                // with another that knows how to handle options)
                let s: String = serde::Deserialize::deserialize(deserializer)?;

                if self.ty == FieldType::Json {
                    let value: serde_json::Value =
                        serde_json::from_str(&s).map_err(de::Error::custom)?;

                    self.seed
                        .deserialize(SomeDeserializer {
                            inner: value.into_deserializer(),
                        })
                        .map_err(de::Error::custom)
                } else {
                    self.seed.deserialize(SomeDeserializer {
                        inner: s.into_deserializer(),
                    })
                }
            }
            FieldType::Timestamp
            | FieldType::Time
            | FieldType::Date
            | FieldType::DateTime
            | FieldType::Integer
            | FieldType::Float => {
                // TODO: make this less hacky and more efficient
                // (there should be a way to wrap this visitor
                // with another that knows how to handle options)
                let s: String = serde::Deserialize::deserialize(deserializer)?;

                if self.ty == FieldType::Integer {
                    let int = s.parse::<i64>().map_err(de::Error::custom)?;
                    self.seed.deserialize(SomeDeserializer {
                        inner: int.into_deserializer(),
                    })
                } else {
                    let float = s.parse::<f64>().map_err(de::Error::custom)?;
                    self.seed.deserialize(SomeDeserializer {
                        inner: float.into_deserializer(),
                    })
                }
            }
            FieldType::Bool => {
                let b = deserializer.deserialize_any(BoolVisitor)?;
                self.seed.deserialize(SomeDeserializer {
                    inner: b.into_deserializer(),
                })
            }
            _ => self
                .seed
                .deserialize(SomeDeserializer {
                    inner: deserializer,
                })
                .map_err(de::Error::custom),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.ty {
            FieldType::String | FieldType::Bytes | FieldType::BigNumeric | FieldType::Numeric => {
                self.seed.deserialize(v.into_deserializer())
            }
            FieldType::Record | FieldType::Json => {
                // TODO: see if we can avoid deserializing into an owned, generic value
                // in order to deserialize nested json.
                let value: serde_json::Value = serde_json::from_str(v).map_err(E::custom)?;
                self.seed
                    .deserialize(value.into_deserializer())
                    .map_err(E::custom)
            }
            FieldType::Integer => {
                let int = v.parse::<i64>().map_err(|err| {
                    de::Error::invalid_type(de::Unexpected::Str(v), &err.to_string().as_str())
                })?;

                self.seed.deserialize(int.into_deserializer())
            }
            FieldType::Float => {
                let int = v.parse::<f64>().map_err(|err| {
                    de::Error::invalid_type(de::Unexpected::Str(v), &err.to_string().as_str())
                })?;

                self.seed.deserialize(int.into_deserializer())
            }
            FieldType::Timestamp | FieldType::DateTime => {
                let timestamp_ms = v.parse::<f64>().map_err(|err| {
                    de::Error::invalid_type(de::Unexpected::Str(v), &err.to_string().as_str())
                })?;

                self.seed.deserialize(timestamp_ms.into_deserializer())
            }
            _ => todo!("visit_str todo: {:?} - '{v}'", self.ty),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.ty {
            // only real string types benefit from getting an owned value, otherwise we should just
            // defer to all encompossing visit_str method
            FieldType::String | FieldType::Bytes | FieldType::Numeric | FieldType::BigNumeric => {
                self.seed.deserialize(v.into_deserializer())
            }
            _ => self.visit_str(&v),
        }
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.ty {
            // similar to the comment in visit_string, only string types will benefit
            // from being able to borrow the actual value, every other type will just
            // be trying to parse it (with the notable exception of nested json records,
            // since the lifetimes match up we can actually parse them without
            // converting to an owned serde_json::Value first)
            FieldType::String | FieldType::Bytes => self
                .seed
                .deserialize(de::value::BorrowedStrDeserializer::new(v)),
            FieldType::Record | FieldType::Json => {
                let mut de = serde_json::Deserializer::from_str(v);
                let de = path_aware_serde::Deserializer::new(&mut de);
                self.seed.deserialize(de).map_err(E::custom)
            }
            _ => self.visit_str(v),
        }
    }
}

pub struct SomeDeserializer<V> {
    inner: V,
}

macro_rules! defer_to_inner_deserialize_fn {
    ($($fn_name:ident),* $(,)?) => {
        $(
            #[inline]
            fn $fn_name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                self.inner.$fn_name(visitor)
            }
        )*
    };
}

impl<'de, D> de::Deserializer<'de> for SomeDeserializer<D>
where
    D: de::Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self.inner)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_option(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_string(visitor)
    }

    defer_to_inner_deserialize_fn! {
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_unit,
        deserialize_map,
        deserialize_seq,
        deserialize_identifier,
        deserialize_ignored_any,
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_struct(name, fields, visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_newtype_struct(name, visitor)
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_unit_struct(name, visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner.deserialize_tuple_struct(name, len, visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }

    /*
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
    */
}

pub struct OptionalValueMapSeed<'a, S>(ValueMapSeed<'a, S>);

impl<'de, S> de::Visitor<'de> for OptionalValueMapSeed<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.expecting(formatter)
    }
}

struct BoolVisitor;

impl<'de> de::Visitor<'de> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a boolean value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.trim() {
            "true" | "TRUE" | "True" => Ok(true),
            "false" | "FALSE" | "False" => Ok(false),
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Str(v),
                &"'true' or 'false'",
            )),
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.trim_ascii() {
            b"true" | b"TRUE" | b"True" => Ok(true),
            b"false" | b"FALSE" | b"False" => Ok(false),
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Bytes(v),
                &"'true' or 'false'",
            )),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(de::Error::invalid_value(
                de::Unexpected::Signed(v),
                &"a valid boolean in integer format, 0 or 1",
            )),
        }
    }
}
