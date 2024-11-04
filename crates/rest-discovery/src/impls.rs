use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use anyhow::anyhow;
use genco::prelude::*;

use crate::ir::{self, Derive, Prim, SharedOrOwned, StdCollection, StructField, TypeRefKind};
use crate::types::{Discovery, Format, RootSchema, Schema, SchemaKind, StringTypeDef, TypeDef};
use crate::{Context, GenerateIr, IntoStatic};

impl Discovery {
    pub fn generate(self, ctx: &mut Context<'_>) -> anyhow::Result<()> {
        ctx.add_module_doc(Cow::Owned(self.description));
        ctx.add_base_url(Cow::Owned(self.base_url));

        for schema in self.schemas.into_values() {
            schema.generate(ctx)?;
        }

        /*
        // todo: add namespacing to fix conflicts between resources
        for (name, schema) in self.parameters {
            (name.into(), schema).generate(ctx)?;
        }

        for resource in self.resources.into_values() {
            for endpoint in resource.methods.into_values() {
                for (param_name, param) in endpoint.parameters {
                    (param_name.into(), param).generate(ctx)?;
                }
            }
        }
        */

        Ok(())
    }
}

fn check_for_misc_optional(s: &str) -> bool {
    s.trim_start_matches(|ch: char| !ch.is_ascii_alphabetic())
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .take(3)
        .any(|word| word.trim().eq_ignore_ascii_case("optional"))
}

fn check_for_desc_markers<const N: usize>(s: &str, markers: [&str; N]) -> bool {
    if markers.is_empty() {
        return false;
    }

    let marker_start = match s.find('[') {
        Some(idx) if s.len() > idx + 1 => idx + 1,
        _ => return false,
    };
    let marker_end = match s[marker_start..].find(']') {
        Some(offset) => marker_start + offset,
        None => return false,
    };

    for found in s[marker_start..marker_end].split_terminator(',') {
        let trimmed = found.trim();

        for m in markers {
            if m.len() == trimmed.len() && m.eq_ignore_ascii_case(trimmed) {
                return true;
            }
        }
    }

    false
}

#[test]
fn test_markers() {
    let example_str = "[Beta] Clustering specification for the table.";
    let expected = ["Beta"];

    assert!(check_for_desc_markers(example_str, expected));

    assert!(check_for_misc_optional("Optional. asdnainfgoeinf"));
}

impl Schema {
    fn is_optional(&self) -> bool {
        const OPTIONAL: [&str; 2] = ["Optional", "Output-only"];
        // required defaults to false, so we'll get lots of false positives if we use it
        // as the ground truth.
        if self.required {
            return false;
        }

        match self.description.as_deref() {
            // check the more standard format first
            Some(desc) => check_for_desc_markers(desc, OPTIONAL) || check_for_misc_optional(desc),
            None => false,
        }
    }

    #[allow(dead_code)]
    fn is_required(&self) -> bool {
        const REQUIRED: [&str; 1] = ["Required"];
        if self.required {
            return true;
        }

        self.description
            .as_deref()
            .map(|s| check_for_desc_markers(s, REQUIRED))
            .unwrap_or_default()
    }
}

impl Format {
    fn build_string_typedef(
        self,
        is_optional: bool,
        ctx: &Context<'_>,
    ) -> SharedOrOwned<ir::TypeRef> {
        let (prim, attr) = match self {
            Format::Double => (Prim::F64, Some(ir::Attr::serde_with("double", is_optional))),
            Format::Uint64 => (Prim::U64, Some(ir::Attr::serde_with("uint64", is_optional))),
            Format::Int64 => (Prim::I64, Some(ir::Attr::serde_with("int64", is_optional))),
            Format::Uint32 => (Prim::U32, Some(ir::Attr::serde_with("uint32", is_optional))),
            Format::Int32 => (Prim::I32, Some(ir::Attr::serde_with("int32", is_optional))),
            Format::Byte => (Prim::get_bytes(ctx), None),
            Format::DateTime => (Prim::Timestamp, None),
            _ => (Prim::String, None),
        };

        ir::TypeRef::builder()
            .is_optional(is_optional)
            .field_attr(attr)
            .kind(prim)
            .build()
            .into()
    }
}

impl<'a> GenerateIr<'a, Rust> for Schema {
    fn generate(self, ctx: &Context<'a>) -> anyhow::Result<SharedOrOwned<ir::TypeRef>> {
        let is_optional = self.is_optional();

        match self.schema_kind {
            SchemaKind::Ref { refer } => match ctx.find_type(&refer) {
                Some(tr) => Ok(SharedOrOwned::Owned(tr)),
                None => Ok(SharedOrOwned::Owned(
                    ir::TypeRef::builder()
                        .is_optional(is_optional)
                        .kind(refer)
                        .build(),
                )),
            },
            SchemaKind::Type { def } => match def {
                TypeDef::Any => Ok(SharedOrOwned::Owned((false, Prim::JsonValue).into())),
                TypeDef::Boolean => Ok(SharedOrOwned::Owned((is_optional, Prim::Bool).into())),
                TypeDef::Integer(_) => Ok(SharedOrOwned::Owned((is_optional, Prim::I64).into())),
                TypeDef::Number(_) => Ok(SharedOrOwned::Owned((is_optional, Prim::F64).into())),
                TypeDef::String { type_def: None } => {
                    Ok(SharedOrOwned::Owned((is_optional, Prim::String).into()))
                }
                TypeDef::Array { items } => {
                    let elem = items.generate(ctx)?;

                    Ok(SharedOrOwned::Owned(ir::TypeRef {
                        is_optional: Cell::new(is_optional),
                        field_attr: Some(ir::Attr::serde_default()),
                        kind: ir::TypeRefKind::Collec(Box::new(StdCollection::Vec {
                            use_cow: ctx.config().use_cowstr,
                            elem,
                        })),
                    }))
                }
                TypeDef::Object { .. } => Err(anyhow!(
                    "objects must be constructed from a name-schema pair {def:#?}"
                )),
                TypeDef::String {
                    type_def: Some(def),
                } => match def {
                    StringTypeDef::String => Ok(SharedOrOwned::Owned(
                        (is_optional, Prim::get_string(ctx)).into(),
                    )),
                    StringTypeDef::Enum(_) => Err(anyhow!(
                        "enum type defs must be constructed from a name-schema pair: {def:#?}"
                    )),
                    StringTypeDef::Numeric { format } => {
                        Ok(format.build_string_typedef(is_optional, ctx))
                    }
                },
            },
        }
    }
}

impl Schema {
    /// Tries to call both [`Schema::try_struct_def`] and [`Schema::try_enum_def`], in that order,
    /// returning the first one that returns [`Ok`].
    fn try_type_def<'a>(
        self,
        name: Cow<'a, str>,
        ctx: &Context<'a>,
    ) -> anyhow::Result<Result<SharedOrOwned<ir::TypeRef>, (Cow<'a, str>, Self)>> {
        match self.try_struct_def(name, ctx)? {
            Ok(def_or_ref) => Ok(Ok(def_or_ref)),
            Err((name, schema)) => match schema.try_enum_def(name, ctx) {
                Ok(refer) => Ok(Ok(refer)),
                Err((name, schema)) => schema.try_array_def(name, ctx),
            },
        }
    }

    fn try_array_def<'a>(
        self,
        name: Cow<'a, str>,
        ctx: &Context<'a>,
    ) -> anyhow::Result<Result<SharedOrOwned<ir::TypeRef>, (Cow<'a, str>, Self)>> {
        if let SchemaKind::Type {
            def: TypeDef::Array { items },
        } = self.schema_kind
        {
            let elem = (name, *items).generate(ctx)?;
            let refer = ir::TypeRef::builder()
                .kind(StdCollection::Vec {
                    elem,
                    use_cow: ctx.config().use_cowstr,
                })
                .build();

            Ok(Ok(SharedOrOwned::Owned(refer)))
        } else {
            Ok(Err((name, self)))
        }
    }

    fn try_struct_def<'a>(
        self,
        mut name: Cow<'a, str>,
        ctx: &Context<'a>,
    ) -> anyhow::Result<Result<SharedOrOwned<ir::TypeRef>, (Cow<'a, str>, Self)>> {
        let (properties, additional_props) = if let SchemaKind::Type {
            def:
                TypeDef::Object {
                    properties,
                    additional_properties,
                },
        } = self.schema_kind
        {
            (properties, additional_properties)
        } else {
            return Ok(Err((name, self)));
        };

        name = Cow::Owned(crate::ir::to_pascal(name));
        if let Some(tr) = ctx.find_type(&*name) {
            return Ok(Ok(ir::SharedOrOwned::Owned(tr)));
        }

        // build the additional props
        let additional = if let Some(value) = additional_props {
            // if we've already defined the type, dont bother regenerating it.
            let refer = if let Some(existing) = ctx.find_type(&*name) {
                SharedOrOwned::Owned(existing)
            } else {
                let value = (name.clone().into(), *value).generate(ctx)?;
                SharedOrOwned::Owned(
                    ir::TypeRef::builder()
                        .is_optional(false)
                        .kind(ir::StdCollection::Map {
                            use_btree: ctx.config().use_btree_map,
                            key: Prim::get_string(ctx),
                            value,
                        })
                        .build(),
                )
            };

            Some(refer)
        } else {
            None
        };

        // if we have no properties, but the additional props, we're a generic map/type alias.
        if properties.is_empty()
            && let Some(value) = additional
        {
            let name = Rc::from(name.into_static());

            let refer = ctx.get_or_insert_type_def(name, |id| {
                ir::TypeDef::builder()
                    .kind(ir::TypeDefKind::Alias(value))
                    .name(id.name.clone())
                    .doc(self.description.map(Cow::from))
                    .id(id)
                    .build()
            });

            return Ok(Ok(refer.into()));
        }

        let mut fields: Vec<ir::StructField<'static>> =
            Vec::with_capacity(properties.len() + additional.is_some() as usize);

        let optional_overrides = ctx.config().override_optional.get(&*name);

        for (field_name, field_schema) in properties {
            // need to ge this before taking the description from the schema.
            let doc = field_schema.description.clone().map(Cow::from);

            let mut kind = (field_name.clone().into(), field_schema).generate(ctx)?;

            if optional_overrides.is_some_and(|fields| fields.contains(&field_name)) {
                kind.is_optional.set(true);
            }

            // check if we're a recurisve type
            if kind
                .name()
                .is_some_and(|ty_name| name.eq_ignore_ascii_case(ty_name))
            {
                if let TypeRefKind::Generated(ref gen) = kind.kind {
                    gen.needs_boxing.set(true);
                }

                // take the inner is_optional, that way we wrap the box in Option, not the inner
                // type.
                let optional = kind.is_optional.take();
                let boxed = ir::TypeRef::builder()
                    .kind(StdCollection::Box(kind))
                    .is_optional(optional)
                    .build();
                kind = SharedOrOwned::Owned(boxed);
            }

            let field: StructField<'static> = StructField::builder()
                .name(field_name)
                .attrs(vec![ir::Attr::builder_into()])
                .doc(doc)
                .ty(kind)
                .build();

            fields.push(field);
        }

        if let Some(addit) = additional {
            let misc_field = StructField::builder()
                .name("misc")
                .doc(Cow::from("misc properties"))
                .attrs(vec![ir::Attr::serde_flatten()])
                .ty(addit)
                .build();

            fields.push(misc_field);
        }

        let refer = ctx.get_or_insert_type_def(Rc::from(name), |id| {
            ir::TypeDef::builder()
                .name(id.name.clone())
                .doc(self.description.map(Cow::from))
                .derive(Derive::DEFAULT_STRUCT)
                .id(id)
                .kind(fields)
                .build()
        });

        Ok(Ok(refer.into()))
    }

    fn try_enum_def<'a>(
        self,
        name: Cow<'a, str>,
        ctx: &Context<'_>,
    ) -> Result<SharedOrOwned<ir::TypeRef>, (Cow<'a, str>, Self)> {
        if let SchemaKind::Type {
            def:
                TypeDef::String {
                    type_def: Some(StringTypeDef::Enum(enum_def)),
                },
        } = self.schema_kind
        {
            let variants = enum_def
                .enum_variants
                .into_iter()
                .zip(enum_def.enum_descriptions.into_iter())
                .map(|(variant, doc)| ir::EnumVariant::from_name_and_doc(variant, doc))
                .collect::<Vec<ir::EnumVariant<'static>>>();

            let refer = ctx.get_or_insert_type_def(Rc::from(name.into_static()), |id| {
                ir::TypeDef::builder()
                    .name(id.name.clone())
                    .doc(self.description.map(Into::into))
                    .derive(Derive::DEFAULT_ENUM)
                    .id(id)
                    .kind(variants)
                    .build()
            });

            Ok(refer.into())
        } else {
            Err((name, self))
        }
    }
}

impl<'a> GenerateIr<'a, Rust> for (Cow<'a, str>, Schema) {
    fn generate(self, ctx: &Context<'a>) -> anyhow::Result<SharedOrOwned<ir::TypeRef>> {
        let (name, schema) = self;

        match schema.try_type_def(name, ctx)? {
            Ok(generated) => Ok(generated),
            Err((_name, schema)) => {
                let refer = schema.generate(ctx)?;
                // if the ref is a primitive type, we may need to add a type alias for everything to
                // typeck properly. this comes up mainly with 'JsonValue' (i.e any
                // valid json/'serde_json::Value')
                /*
                if refer.is_primitive() {
                    Ok(ctx.insert_named_ref(name, refer))
                } else {
                    Ok(refer)
                }
                */
                Ok(refer)
            }
        }
    }
}

impl<'a> GenerateIr<'a, Rust> for RootSchema {
    fn generate(self, ctx: &Context<'a>) -> anyhow::Result<SharedOrOwned<ir::TypeRef>> {
        self.into_id_schema_pair().generate(ctx)
    }
}
