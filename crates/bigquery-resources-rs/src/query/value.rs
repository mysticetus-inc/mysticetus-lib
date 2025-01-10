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
                        value = Some(map.next_value_seed(ValueSeed {
                            mode: self.mode,
                            ty: self.ty,
                            field_name: self.field_name,
                            seed,
                        })?)
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
            "a {:?} {:?} encoded via the BigQuery REST API",
            self.mode, self.ty
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
        deserialize_according_to_type(self, deserializer)
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
            FieldType::Timestamp | FieldType::DateTime => {
                let timestamp_ms = v.parse::<f64>().map_err(|err| {
                    de::Error::invalid_type(de::Unexpected::Str(v), &err.to_string().as_str())
                })?;

                self.seed.deserialize(timestamp_ms.into_deserializer())
            }
            _ => todo!("visit_str todo: {:?}", self.ty),
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