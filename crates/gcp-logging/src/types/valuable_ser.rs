use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple, SerializeTupleStruct,
};
use valuable::Fields;

pub struct SerializeValue<'a>(pub valuable::Value<'a>);

impl serde::Serialize for SerializeValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            valuable::Value::Bool(b) => serializer.serialize_bool(b),
            valuable::Value::Char(ch) => serializer.serialize_char(ch),
            valuable::Value::F32(f) => serializer.serialize_f32(f),
            valuable::Value::F64(f) => serializer.serialize_f64(f),
            valuable::Value::I8(i) => serializer.serialize_i8(i),
            valuable::Value::I16(i) => serializer.serialize_i16(i),
            valuable::Value::I32(i) => serializer.serialize_i32(i),
            valuable::Value::I64(i) => serializer.serialize_i64(i),
            valuable::Value::I128(i) => serializer.serialize_i128(i),
            valuable::Value::Isize(i) => serializer.serialize_i64(i as i64),
            valuable::Value::String(s) => serializer.serialize_str(s),
            valuable::Value::U8(u) => serializer.serialize_u8(u),
            valuable::Value::U16(u) => serializer.serialize_u16(u),
            valuable::Value::U32(u) => serializer.serialize_u32(u),
            valuable::Value::U64(u) => serializer.serialize_u64(u),
            valuable::Value::U128(u) => serializer.serialize_u128(u),
            valuable::Value::Usize(u) => serializer.serialize_u64(u as u64),
            valuable::Value::Path(path) => match path.to_str() {
                Some(s) => serializer.serialize_str(s),
                None => serializer.collect_str(&path.display()),
            },
            valuable::Value::Error(error) => SerializeError(error).serialize(serializer),
            valuable::Value::Listable(listable) => {
                SerializeListable(listable).serialize(serializer)
            }
            valuable::Value::Mappable(mappable) => {
                SerializeMappable(mappable).serialize(serializer)
            }
            valuable::Value::Structable(structable) => {
                SerializeStructable(structable).serialize(serializer)
            }
            valuable::Value::Enumerable(enumerable) => todo!(),
            valuable::Value::Tuplable(tuplable) => {
                SerializeTuplable(tuplable).serialize(serializer)
            }
            valuable::Value::Unit => serializer.serialize_unit(),
            _ => todo!("TODO: new valuable::Value variant - {:?}", self.0),
        }
    }
}

macro_rules! visit_serialize {
    ($value:expr; $visitor:expr) => {{
        let mut visitor = { $visitor };

        valuable::Valuable::visit(&$value, &mut visitor);

        if let Some(error) = visitor.error {
            return Err(error);
        }

        visitor.inner.end()
    }};
}

pub struct SerializeError<'a>(pub &'a (dyn std::error::Error + 'static));

impl serde::Serialize for SerializeError<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

pub struct SerializeListable<'a>(pub &'a dyn valuable::Listable);

impl serde::Serialize for SerializeListable<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let size = match self.0.size_hint() {
            (low, Some(high)) if low == high => Some(low),
            _ => None,
        };

        let inner = serializer.serialize_seq(size)?;

        visit_serialize!(self.0; VisitSerializeSeq { inner, error: None })
    }
}

pub struct VisitSerializeSeq<S: serde::ser::SerializeSeq> {
    inner: S,
    error: Option<S::Error>,
}

impl<S: serde::ser::SerializeSeq> valuable::Visit for VisitSerializeSeq<S> {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if self.error.is_none() {
            self.error = self.inner.serialize_element(&SerializeValue(value)).err();
        }
    }

    fn visit_primitive_slice(&mut self, slice: valuable::Slice<'_>) {
        if self.error.is_none() {
            self.error = match slice {
                valuable::Slice::Bool(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::Char(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::F32(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::F64(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::I8(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::I16(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::I32(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::I64(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::I128(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::Isize(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::Str(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::String(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::U8(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::U16(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::U32(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::U64(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::U128(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::Usize(items) => self.inner.serialize_element(items).err(),
                valuable::Slice::Unit(items) => self.inner.serialize_element(items).err(),
                _ => None,
            };
        }
    }

    fn visit_entry(&mut self, key: valuable::Value<'_>, value: valuable::Value<'_>) {
        if self.error.is_none() {
            self.error = self
                .inner
                .serialize_element(&(SerializeValue(key), SerializeValue(value)))
                .err();
        }
    }

    fn visit_named_fields(&mut self, named_values: &valuable::NamedValues<'_>) {
        if self.error.is_some() {
            return;
        }
        for (field, value) in named_values {
            self.error = self
                .inner
                .serialize_element(&(field.name(), SerializeValue(*value)))
                .err();

            if self.error.is_some() {
                return;
            }
        }
    }

    fn visit_unnamed_fields(&mut self, values: &[valuable::Value<'_>]) {
        if self.error.is_some() {
            return;
        }

        for value in values {
            self.visit_value(*value);

            if self.error.is_some() {
                return;
            }
        }
    }
}

pub struct SerializeTuplable<'a>(pub &'a dyn valuable::Tuplable);

impl serde::Serialize for SerializeTuplable<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let size = match self.0.definition() {
            valuable::TupleDef::Static { fields, .. } => Some(fields),
            valuable::TupleDef::Dynamic {
                fields: (low, Some(high)),
                ..
            } if low == high => Some(low),
            _ => None,
        };

        let inner = serializer.serialize_seq(size)?;

        visit_serialize!(self.0; VisitSerializeSeq { inner, error: None })
    }
}

pub struct SerializeMappable<'a>(pub &'a dyn valuable::Mappable);

impl serde::Serialize for SerializeMappable<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let size = match self.0.size_hint() {
            (low, Some(high)) if low == high => Some(low),
            _ => None,
        };

        let inner = serializer.serialize_map(size)?;

        visit_serialize!(self.0; VisitSerializeMap {
            inner,
            error: None,
            unnamed: 0,
        })
    }
}

pub struct VisitSerializeMap<M: serde::ser::SerializeMap> {
    inner: M,
    unnamed: usize,
    error: Option<M::Error>,
}

impl<S: serde::ser::SerializeMap> valuable::Visit for VisitSerializeMap<S> {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if self.error.is_none() {
            let idx = self.unnamed;
            self.unnamed += 1;

            self.error = self
                .inner
                .serialize_entry(&format_args!("__unnamed_{idx}"), &SerializeValue(value))
                .err();
        }
    }

    fn visit_entry(&mut self, key: valuable::Value<'_>, value: valuable::Value<'_>) {
        if self.error.is_none() {
            self.error = self
                .inner
                .serialize_entry(&SerializeValue(key), &SerializeValue(value))
                .err();
        }
    }

    fn visit_named_fields(&mut self, named_values: &valuable::NamedValues<'_>) {
        if self.error.is_some() {
            return;
        }

        for (field, value) in named_values {
            self.error = self
                .inner
                .serialize_entry(field.name(), &SerializeValue(*value))
                .err();

            if self.error.is_some() {
                return;
            }
        }
    }

    fn visit_unnamed_fields(&mut self, values: &[valuable::Value<'_>]) {
        if self.error.is_some() {
            return;
        }

        for value in values {
            self.visit_value(*value);

            if self.error.is_some() {
                return;
            }
        }
    }
}

pub struct SerializeStructable<'a>(pub &'a dyn valuable::Structable);

impl serde::Serialize for SerializeStructable<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let def = self.0.definition();
        match def {
            valuable::StructDef::Static {
                name,
                fields: valuable::Fields::Named(named),
                ..
            } => {
                let inner = serializer.serialize_struct(name, named.len())?;

                visit_serialize!(self.0; VisitSerializeStruct {
                    inner,
                    error: None,
                    fields: named.iter(),
                })
            }
            valuable::StructDef::Static {
                name,
                fields: valuable::Fields::Unnamed(count),
                ..
            } => {
                let inner = serializer.serialize_tuple_struct(name, count)?;

                visit_serialize!(self.0; VisitSerializeTupleStruct {
                    inner,
                    error: None,
                })
            }
            _ => match def.fields() {
                valuable::Fields::Named(named) => {
                    let fields = def.fields();
                    let inner = serializer.serialize_map(Some(fields.len()))?;

                    visit_serialize!(self.0; VisitSerializeStructMap {
                        inner,
                        error: None,
                        fields: named.iter(),
                    })
                }
                valuable::Fields::Unnamed(count) => {
                    let inner = serializer.serialize_tuple(*count)?;
                    visit_serialize!(self.0; VisitSerializeTuple {
                        inner,
                        error: None,
                    })
                }
            },
        }
    }
}

pub struct VisitSerializeStruct<S: SerializeStruct> {
    inner: S,
    fields: std::slice::Iter<'static, valuable::NamedField<'static>>,
    error: Option<S::Error>,
}

impl<S: SerializeStruct> valuable::Visit for VisitSerializeStruct<S> {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if self.error.is_none() {
            let field = self
                .fields
                .next()
                .expect("serializing more fields than expected");

            self.error = self
                .inner
                .serialize_field(field.name(), &SerializeValue(value))
                .err();
        }
    }

    fn visit_entry(&mut self, _key: valuable::Value<'_>, value: valuable::Value<'_>) {
        self.visit_value(value);
    }

    fn visit_unnamed_fields(&mut self, values: &[valuable::Value<'_>]) {
        if self.error.is_some() {
            return;
        }

        for (field, value) in self.fields.by_ref().zip(values) {
            self.error = self
                .inner
                .serialize_field(field.name(), &SerializeValue(*value))
                .err();

            if self.error.is_some() {
                return;
            }
        }
    }

    fn visit_named_fields(&mut self, named_values: &valuable::NamedValues<'_>) {
        if self.error.is_some() {
            return;
        }

        for (field, (_, value)) in self.fields.by_ref().zip(named_values.iter()) {
            self.error = self
                .inner
                .serialize_field(field.name(), &SerializeValue(*value))
                .err();

            if self.error.is_some() {
                return;
            }
        }
    }
}

pub struct VisitSerializeStructMap<'a, S: SerializeMap> {
    inner: S,
    fields: std::slice::Iter<'a, valuable::NamedField<'a>>,
    error: Option<S::Error>,
}

impl<S: SerializeMap> valuable::Visit for VisitSerializeStructMap<'_, S> {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if self.error.is_none() {
            let field = self
                .fields
                .next()
                .expect("serializing more fields than expected");

            self.error = self
                .inner
                .serialize_entry(field.name(), &SerializeValue(value))
                .err();
        }
    }

    fn visit_entry(&mut self, key: valuable::Value<'_>, value: valuable::Value<'_>) {
        if self.error.is_none() {
            self.fields.next();

            self.error = self
                .inner
                .serialize_entry(&SerializeValue(key), &SerializeValue(value))
                .err();
        }
    }

    fn visit_unnamed_fields(&mut self, values: &[valuable::Value<'_>]) {
        if self.error.is_some() {
            return;
        }

        for (field, value) in self.fields.by_ref().zip(values) {
            self.error = self
                .inner
                .serialize_entry(field.name(), &SerializeValue(*value))
                .err();

            if self.error.is_some() {
                return;
            }
        }
    }

    fn visit_named_fields(&mut self, named_values: &valuable::NamedValues<'_>) {
        if self.error.is_some() {
            return;
        }

        for (field, value) in named_values {
            self.error = self
                .inner
                .serialize_entry(field.name(), &SerializeValue(*value))
                .err();

            self.fields.next();

            if self.error.is_some() {
                return;
            }
        }
    }
}

pub struct VisitSerializeTupleStruct<S: SerializeTupleStruct> {
    inner: S,
    error: Option<S::Error>,
}

impl<S: SerializeTupleStruct> valuable::Visit for VisitSerializeTupleStruct<S> {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if self.error.is_none() {
            self.error = self.inner.serialize_field(&SerializeValue(value)).err();
        }
    }
}

pub struct VisitSerializeTuple<S: SerializeTuple> {
    inner: S,
    error: Option<S::Error>,
}

impl<S: SerializeTuple> valuable::Visit for VisitSerializeTuple<S> {
    fn visit_value(&mut self, value: valuable::Value<'_>) {
        if self.error.is_none() {
            self.error = self.inner.serialize_element(&SerializeValue(value)).err();
        }
    }
}

pub struct SerializeEnumerable<'a>(pub &'a dyn valuable::Enumerable);

impl serde::Serialize for SerializeEnumerable<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        fn try_extract_static_parts(
            enum_def: &valuable::EnumDef<'_>,
            variant: &valuable::Variant<'_>,
        ) -> Option<(&'static str, u32, &'static str, &'static Fields<'static>)> {
            let (
                valuable::EnumDef::Static { name, variants, .. },
                valuable::Variant::Static(variant),
            ) = (enum_def, variant)
            else {
                return None;
            };

            fn is_same(a: &valuable::VariantDef<'_>, b: &valuable::VariantDef<'_>) -> bool {
                if a.name() != b.name() {
                    return false;
                }

                let (a_named, b_named) = match (a.fields(), b.fields()) {
                    (Fields::Named(a_named), Fields::Named(b_named)) => (a_named, b_named),
                    (Fields::Named(_), Fields::Unnamed(_))
                    | (Fields::Unnamed(_), Fields::Named(_)) => return false,
                    (Fields::Unnamed(a_count), Fields::Unnamed(b_count)) => {
                        return a_count == b_count;
                    }
                };

                if a_named.len() != b_named.len() {
                    return false;
                }

                a_named
                    .iter()
                    .zip(b_named.iter())
                    .all(|(a, b)| a.name() == b.name())
            }

            let pos = variants.iter().position(|var| is_same(var, variant))?;

            Some((name, pos as u32, variant.name(), variant.fields()))
        }

        let enum_def = self.0.definition();
        let variant_def = self.0.variant();

        match try_extract_static_parts(&enum_def, &variant_def) {
            Some((name, variant_index, variant_name, Fields::Named(named))) => {
                let inner = serializer.serialize_struct_variant(
                    name,
                    variant_index,
                    variant_name,
                    named.len(),
                )?;

                todo!()
            }
            Some((name, variant_index, variant_name, Fields::Unnamed(count))) => {
                let inner = serializer.serialize_tuple_variant(
                    name,
                    variant_index,
                    variant_name,
                    *count,
                )?;

                todo!()
            }
            None => match variant_def.fields() {
                Fields::Named(named) => {
                    let inner = serializer.serialize_map(Some(named.len()))?;
                    todo!()
                }
                Fields::Unnamed(named) => todo!(),
            },
        }
    }
}
