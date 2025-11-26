//! An "IR" for generating types.
//!
//! These types implement [`genco::prelude::FormatInto`], so to generate code, all the
//! discovery docuemnt needs to do is generate these types.
//!
//! Since the types generated will all be relatively simple, these types intentionally leave out
//! __a lot__ of rusts possible values (no support for generics, >1 lifetime, enums that contain
//! values, etc.)
use std::borrow::Cow;
use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;

use genco::prelude::*;
use typed_builder::TypedBuilder;

use super::{Context, GenerateCode, IntoStatic};
use crate::type_defs::TypeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SharedOrOwned<T> {
    Owned(T),
    Shared(Rc<T>),
}

impl<T> From<T> for SharedOrOwned<T> {
    fn from(owned: T) -> Self {
        Self::Owned(owned)
    }
}

impl<T> From<Rc<T>> for SharedOrOwned<T> {
    fn from(shared: Rc<T>) -> Self {
        Self::Shared(shared)
    }
}

impl<T> AsRef<T> for SharedOrOwned<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Owned(owned) => owned,
            Self::Shared(shared) => &**shared,
        }
    }
}

impl<T> Deref for SharedOrOwned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Self::Owned(owned) => owned,
            Self::Shared(shared) => &**shared,
        }
    }
}

static KEYWORD_REPL_MAP: phf::Map<&'static str, (&'static str, &'static str)> = phf::phf_map! {
    "type" => ("ty", "#[serde(rename = \"type\")]"),
};

fn to_snake<'a, S: AsRef<str>>(s: S) -> (Cow<'a, str>, Option<&'static str>) {
    use convert_case::{Case, Casing};

    match KEYWORD_REPL_MAP.get(s.as_ref()) {
        Some((key, attr)) => (Cow::Borrowed(key), Some(attr)),
        None => (Cow::Owned(s.as_ref().to_case(Case::Snake)), None),
    }
}

pub(crate) fn to_pascal<S: AsRef<str>>(s: S) -> String {
    use convert_case::{Case, Casing};

    s.as_ref().to_case(Case::Pascal)
}

impl From<(bool, Prim)> for TypeRef {
    fn from(p: (bool, Prim)) -> Self {
        TypeRef {
            field_attr: None,
            is_optional: Cell::new(p.0),
            kind: TypeRefKind::Prim(p.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prim {
    F64,
    F32,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    Bool,
    Char,
    Str,
    CowStr,
    CowBytes,
    Bytes,
    ByteVec,
    String,
    Timestamp,
    Duration,
    JsonValue,
}

impl Prim {
    pub fn has_lifetime(&self) -> bool {
        matches!(*self, Self::Str | Self::CowStr | Self::CowBytes)
    }

    pub fn has_default(&self) -> bool {
        !matches!(
            self,
            Self::CowStr | Self::CowBytes | Self::Timestamp | Self::Duration | Self::Char
        )
    }

    pub fn get_bytes(ctx: &Context<'_>) -> Self {
        if ctx.config().use_bytes {
            Self::Bytes
        } else {
            Self::ByteVec
        }
    }

    pub fn get_string(ctx: &Context<'_>) -> Self {
        if ctx.config().use_cowstr {
            Self::CowStr
        } else {
            Self::String
        }
    }
}

impl FormatInto<Rust> for Prim {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        match self {
            Self::F64 => tokens.append("f64"),
            Self::F32 => tokens.append("f32"),
            Self::I8 => tokens.append("i8"),
            Self::I16 => tokens.append("i16"),
            Self::I32 => tokens.append("i32"),
            Self::I64 => tokens.append("i64"),
            Self::I128 => tokens.append("i128"),
            Self::Isize => tokens.append("isize"),
            Self::U8 => tokens.append("u8"),
            Self::U16 => tokens.append("u16"),
            Self::U32 => tokens.append("u32"),
            Self::U64 => tokens.append("u64"),
            Self::U128 => tokens.append("u128"),
            Self::Usize => tokens.append("usize"),
            Self::Bool => tokens.append("bool"),
            Self::Char => tokens.append("char"),
            Self::Str => tokens.append("&'a str"),
            Self::String => tokens.append("::std::string::String"),
            Self::CowStr => tokens.append("::std::borrow::Cow<'a, str>"),
            Self::CowBytes => tokens.append("::std::borrow::Cow<'a, [u8]>"),
            Self::Timestamp => tokens.append("::timestamp::Timestamp"),
            Self::Duration => tokens.append("::timestamp::Duration"),
            Self::ByteVec => tokens.append("::std::vec::Vec<u8>"),
            Self::Bytes => tokens.append("::bytes::Bytes"),
            Self::JsonValue => tokens.append("::serde_json::Value"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StdCollection {
    Map {
        use_btree: bool,
        key: Prim,
        value: SharedOrOwned<TypeRef>,
    },
    Vec {
        use_cow: bool,
        elem: SharedOrOwned<TypeRef>,
    },
    Box(SharedOrOwned<TypeRef>),
}

impl StdCollection {
    pub fn has_default(&self, ctx: &Context<'_>) -> Option<bool> {
        match self {
            Self::Map { .. } | Self::Vec { .. } => Some(true),
            Self::Box(refer) => refer.has_default(ctx),
        }
    }
    pub fn has_lifetime(&self, ctx: &Context<'_>) -> Option<bool> {
        match self {
            Self::Map { key, value, .. } => {
                if key.has_lifetime() {
                    Some(true)
                } else {
                    value.has_lifetime(ctx).map(|v| v || key.has_lifetime())
                }
            }
            Self::Vec { use_cow: true, .. } => Some(true),
            Self::Vec { elem, .. } => elem.has_lifetime(ctx),
            Self::Box(inner) => inner.has_lifetime(ctx),
        }
    }
}

impl GenerateCode<Rust> for StdCollection {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<Rust>) {
        match self {
            Self::Map {
                use_btree,
                key,
                value,
            } => {
                let map_variant = if *use_btree {
                    "::std::collections::BTreeMap"
                } else {
                    "::std::collections::HashMap"
                };

                quote_in! { *tokens => $map_variant<$(*key), $(value.as_format_into(ctx))> }
            }
            Self::Vec { use_cow, elem } => {
                if *use_cow {
                    quote_in! { *tokens => ::std::borrow::Cow<'a, [$(elem.as_format_into(ctx))]> }
                } else {
                    quote_in! { *tokens => ::std::vec::Vec<$(elem.as_format_into(ctx))> }
                }
            }
            Self::Box(inner) => {
                quote_in! { *tokens => ::std::boxed::Box<$(inner.as_format_into(ctx))> }
            }
        }
    }
}

impl GenerateCode<Rust> for TypeRef {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<Rust>) {
        if self.is_optional.get() {
            quote_in! { *tokens =>  ::std::option::Option<$(self.kind.as_format_into(ctx))> };
        } else {
            self.kind.generate_code(ctx, tokens);
        }
    }
}

impl GenerateCode<Rust> for TypeRefKind {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<Rust>) {
        match self {
            Self::Generated(gener) => gener.format_into(tokens),
            Self::Prim(prim) => prim.format_into(tokens),
            Self::Collec(collec) => collec.generate_code(ctx, tokens),
            Self::Undefined(name) => match ctx.find_type(name) {
                Some(found) => found.generate_code(ctx, tokens),
                None => panic!("cant find type at codegen time for {name}"),
            },
        }
    }
}

impl FormatInto<Rust> for &GeneratedTypeRef {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        quote_in! { *tokens => $(to_pascal(&*self.name))$(self.lifetime.then_some("<'a>")) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder)]
pub struct TypeRef {
    #[builder(default, setter(into))]
    pub is_optional: Cell<bool>,
    #[builder(default, setter(into))]
    pub field_attr: Option<Attr<'static>>,
    #[builder(setter(into))]
    pub kind: TypeRefKind,
}

impl TypeRef {
    pub fn name(&self) -> Option<&str> {
        match &self.kind {
            TypeRefKind::Undefined(name) => Some(name.as_str()),
            TypeRefKind::Generated(gener) => Some(&*gener.name),
            _ => None,
        }
    }

    pub fn has_default(&self, ctx: &Context<'_>) -> Option<bool> {
        if self.is_optional.get() {
            return Some(true);
        }

        match &self.kind {
            TypeRefKind::Generated(gener) => Some(gener.has_default),
            TypeRefKind::Prim(p) => Some(p.has_default()),
            TypeRefKind::Collec(c) => c.has_default(ctx),
            TypeRefKind::Undefined(name) => {
                ctx.find_type(name).and_then(|def| def.has_default(ctx))
            }
        }
    }

    pub fn is_primitive(&self) -> bool {
        match &self.kind {
            TypeRefKind::Prim(_) => true,
            _ => false,
        }
    }

    pub fn name_or_rc(&self) -> Option<Result<&Rc<str>, &str>> {
        match &self.kind {
            TypeRefKind::Undefined(name) => Some(Err(name.as_str())),
            TypeRefKind::Generated(gener) => Some(Ok(&gener.name)),
            _ => None,
        }
    }

    pub fn has_lifetime(&self, ctx: &Context<'_>) -> Option<bool> {
        match &self.kind {
            TypeRefKind::Generated(gener) => Some(gener.lifetime),
            TypeRefKind::Prim(prim) => Some(prim.has_lifetime()),
            TypeRefKind::Collec(collec) => collec.has_lifetime(ctx),
            TypeRefKind::Undefined(name) => {
                ctx.find_type(name).and_then(|def| def.has_lifetime(ctx))
            }
        }
    }

    pub fn find_if_undef(self, context: &Context<'_>) -> SharedOrOwned<Self> {
        if let TypeRefKind::Undefined(name) = &self.kind {
            match context.find_type(name) {
                Some(found) => found.into(),
                None => panic!("cant find typedef {self:#?}"),
            }
        } else {
            self.into()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRefKind {
    Generated(GeneratedTypeRef),
    Prim(Prim),
    Collec(Box<StdCollection>),
    Undefined(String),
}

impl From<Prim> for TypeRefKind {
    fn from(p: Prim) -> Self {
        Self::Prim(p)
    }
}

impl From<GeneratedTypeRef> for TypeRefKind {
    fn from(r: GeneratedTypeRef) -> Self {
        Self::Generated(r)
    }
}

impl From<StdCollection> for TypeRefKind {
    fn from(c: StdCollection) -> Self {
        Self::Collec(Box::new(c))
    }
}

impl From<String> for TypeRefKind {
    fn from(name: String) -> Self {
        Self::Undefined(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder)]
pub struct TypeDef<'a> {
    pub id: TypeId,
    #[builder(setter(into))]
    pub doc: Option<Cow<'a, str>>,
    #[builder(default, setter(into))]
    pub derive: Vec<Derive<'a>>,
    #[builder(setter(into))]
    pub name: Rc<str>,
    #[builder(default, setter(into))]
    pub attrs: Vec<Attr<'a>>,
    #[builder(setter(into))]
    pub kind: TypeDefKind<'a>,
}

impl<'a> TypeDef<'a> {
    pub fn has_lifetime(&self, ctx: &Context<'_>) -> Option<bool> {
        match &self.kind {
            TypeDefKind::Alias(alias) => alias.has_lifetime(ctx),
            // needs to be fixed if we start allowing enums with values.
            TypeDefKind::Enum(_) => Some(false),
            TypeDefKind::Struct(fields) => fields.iter().find_map(|field| field.has_lifetime(ctx)),
        }
    }

    pub fn can_be_default(&self, ctx: &Context<'_>) -> Option<bool> {
        match &self.kind {
            TypeDefKind::Alias(alias) => alias.has_default(ctx),
            TypeDefKind::Enum(_) => Some(false),
            TypeDefKind::Struct(fields) => {
                for field in fields.iter() {
                    match field.ty.has_default(ctx) {
                        Some(true) => (),
                        Some(false) => return Some(false),
                        None => return None,
                    }
                }

                Some(true)
            }
        }
    }

    pub fn check_for_boxing(&self, ctx: &Context<'_>) {
        let TypeDefKind::Struct(ref fields) = self.kind else {
            return;
        };

        for field in fields {
            field.check_for_boxing(&self.name, ctx);
        }
    }

    pub fn is_primitive(&self) -> bool {
        match &self.kind {
            TypeDefKind::Alias(alias) => alias.is_primitive(),
            // all other variants are composed types, therefore cannot be primitive
            _ => false,
        }
    }

    pub(super) fn as_type_ref(&'a self, ctx: &Context<'_>) -> TypeRef {
        TypeRef {
            is_optional: Cell::new(false),
            field_attr: None, // this only applies to struct fields, not type defs
            kind: TypeRefKind::Generated(GeneratedTypeRef {
                id: self.id.clone(),
                name: Rc::clone(&self.name),
                needs_boxing: Cell::new(false),
                lifetime: self.has_lifetime(ctx).unwrap_or_default(),
                has_default: self.can_be_default(ctx).unwrap_or_default(),
            }),
        }
    }
}

impl<'a> IntoStatic<'a> for TypeDef<'a> {
    type Static = TypeDef<'static>;

    fn into_static(self) -> Self::Static {
        TypeDef {
            id: self.id,
            doc: self.doc.map(IntoStatic::into_static),
            derive: self.derive.into_static(),
            name: self.name,
            attrs: self.attrs.into_static(),
            kind: self.kind.into_static(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Derive<'a> {
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    Default,
    PartialOrd,
    Ord,
    Hash,
    TypedBuilder,
    Other(Cow<'a, str>),
}

impl Derive<'static> {
    pub const DEFAULT_STRUCT: &'static [Self] = &[
        Self::Deserialize,
        Self::Serialize,
        Self::TypedBuilder,
        Self::Debug,
        Self::PartialEq,
        Self::Clone,
    ];
    pub const DEFAULT_ENUM: &'static [Self] = &[
        Self::Deserialize,
        Self::Serialize,
        Self::Debug,
        Self::PartialEq,
        Self::Clone,
    ];
}

impl<'a> IntoStatic<'a> for Derive<'a> {
    type Static = Derive<'static>;
    fn into_static(self) -> Self::Static {
        match self {
            Self::Other(other) => Derive::Other(other.into_static()),
            Self::Deserialize => Derive::Deserialize,
            Self::Serialize => Derive::Serialize,
            Self::PartialEq => Derive::PartialEq,
            Self::Eq => Derive::Eq,
            Self::Debug => Derive::Debug,
            Self::Default => Derive::Default,
            Self::Clone => Derive::Clone,
            Self::PartialOrd => Derive::PartialOrd,
            Self::Ord => Derive::Ord,
            Self::Hash => Derive::Hash,
            Self::TypedBuilder => Derive::TypedBuilder,
        }
    }
}

impl FormatInto<Rust> for &Derive<'_> {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        match self {
            Derive::Deserialize => tokens.append("::serde::Deserialize"),
            Derive::Serialize => tokens.append("::serde::Serialize"),
            Derive::Debug => tokens.append("Debug"),
            Derive::PartialEq => tokens.append("PartialEq"),
            Derive::Clone => tokens.append("Clone"),
            Derive::Eq => tokens.append("Eq"),
            Derive::Default => tokens.append("Default"),
            Derive::PartialOrd => tokens.append("PartialOrd"),
            Derive::Ord => tokens.append("Ord"),
            Derive::Hash => tokens.append("Hash"),
            Derive::TypedBuilder => tokens.append("::typed_builder::TypedBuilder"),
            Derive::Other(other) => tokens.append(&**other),
        }
    }
}

impl GenerateCode<Rust> for TypeDef<'_> {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<Rust>) {
        self.check_for_boxing(ctx);

        quote_in! { *tokens =>
            $(ctx.doc_opt(self.doc.as_ref(), 0))
        }

        let name = to_pascal(&*self.name);

        match &self.kind {
            TypeDefKind::Alias(ty_ref) => {
                let lt = match ty_ref.has_lifetime(ctx) {
                    Some(b) => b.then_some("<'a>"),
                    None => panic!("{} cant determine lifetime check: {self:#?}", self.name),
                };
                quote_in! { *tokens =>
                    pub type $name$lt = $(ty_ref.as_format_into(ctx));
                    $['\n']
                }
            }
            TypeDefKind::Enum(variants) => {
                quote_in! { *tokens =>
                    $(for attr in self.attrs.iter() => $attr $['\r'])
                    #[derive($(for derive in self.derive.iter() join (, ) => $derive))]
                    $(if let Some(derives) = ctx.get_derives(&name).filter(|der| !der.is_empty()) {
                        #[derive($(for der in derives join (, ) => $der))]
                    })
                    pub enum $name {
                        $(variants.as_format_into(ctx))
                    }
                    $['\n']
                }
            }
            TypeDefKind::Struct(fields) => {
                let lt = fields
                    .iter()
                    .filter_map(|field| field.has_lifetime(ctx))
                    .any(std::convert::identity)
                    .then_some("<'a>");

                let default = self.can_be_default(ctx).unwrap_or_default();

                quote_in! { *tokens =>
                    $(for attr in self.attrs.iter() => $attr $['\r'])
                    #[derive($(for derive in self.derive.iter() join (, ) => $derive))]
                    $(if let Some(derives) = ctx.get_derives(&name).filter(|der| !der.is_empty()) {
                        #[derive($(for der in derives join (, ) => $der))]
                    })
                    $(if default => #[derive($(&Derive::Default))])
                    #[serde(rename_all = "camelCase")]
                    pub struct $name$(lt) {
                        $(fields.as_format_into(ctx))
                    }
                    $['\n']
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTypeRef {
    pub id: TypeId,
    pub name: Rc<str>,
    pub has_default: bool,
    pub needs_boxing: Cell<bool>,
    pub lifetime: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDefKind<'a> {
    Alias(SharedOrOwned<TypeRef>),
    Enum(Vec<EnumVariant<'a>>),
    Struct(Vec<StructField<'a>>),
}

impl<'a> IntoStatic<'a> for TypeDefKind<'a> {
    type Static = TypeDefKind<'static>;

    fn into_static(self) -> Self::Static {
        match self {
            Self::Alias(alias) => TypeDefKind::Alias(alias),
            Self::Enum(variants) => TypeDefKind::Enum(variants.into_static()),
            Self::Struct(fields) => TypeDefKind::Struct(fields.into_static()),
        }
    }
}

impl From<TypeRef> for TypeDefKind<'static> {
    fn from(r: TypeRef) -> Self {
        Self::Alias(SharedOrOwned::Owned(r))
    }
}
impl From<Rc<TypeRef>> for TypeDefKind<'static> {
    fn from(r: Rc<TypeRef>) -> Self {
        Self::Alias(SharedOrOwned::Shared(r))
    }
}

impl From<SharedOrOwned<TypeRef>> for TypeDefKind<'static> {
    fn from(either: SharedOrOwned<TypeRef>) -> Self {
        Self::Alias(either)
    }
}

impl<'a> From<Vec<EnumVariant<'a>>> for TypeDefKind<'a> {
    fn from(vars: Vec<EnumVariant<'a>>) -> Self {
        Self::Enum(vars)
    }
}

impl<'a> From<Vec<StructField<'a>>> for TypeDefKind<'a> {
    fn from(fields: Vec<StructField<'a>>) -> Self {
        Self::Struct(fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct Attr<'a> {
    /// corresponds to the '#[serde()]' part of an attribute
    #[builder(setter(into))]
    pub name: Cow<'a, str>,
    /// Corresponds to arguments within the attribute.
    pub args: Vec<AttrArg<'a>>,
}

impl Attr<'static> {
    pub(crate) fn serde_flatten() -> Self {
        Self {
            name: "serde".into(),
            args: vec![AttrArg {
                arg_name: "flatten".into(),
                arg_value: None,
            }],
        }
    }

    pub(crate) fn serde_default() -> Self {
        Self {
            name: "serde".into(),
            args: vec![AttrArg {
                arg_name: "default".into(),
                arg_value: None,
            }],
        }
    }

    pub(crate) fn builder_into() -> Self {
        Self {
            name: "builder".into(),
            args: vec![AttrArg {
                arg_name: "setter(into)".into(),
                arg_value: None,
            }],
        }
    }

    pub(crate) fn serde_with(fmt: &str, optional: bool) -> Self {
        let value = if optional {
            format!("\"with::{fmt}::option\"")
        } else {
            format!("\"with::{fmt}\"")
        };

        Self {
            name: "serde".into(),
            args: vec![AttrArg {
                arg_name: "with".into(),
                arg_value: Some(value.into()),
            }],
        }
    }
}

impl<'a> IntoStatic<'a> for Attr<'a> {
    type Static = Attr<'static>;

    fn into_static(self) -> Self::Static {
        Attr {
            name: self.name.into_static(),
            args: self.args.into_static(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct AttrArg<'a> {
    #[builder(setter(into))]
    pub arg_name: Cow<'a, str>,
    #[builder(setter(into, strip_option))]
    pub arg_value: Option<Cow<'a, str>>,
}

impl<'a> IntoStatic<'a> for AttrArg<'a> {
    type Static = AttrArg<'static>;

    fn into_static(self) -> Self::Static {
        AttrArg {
            arg_name: self.arg_name.into_static(),
            arg_value: self.arg_value.map(IntoStatic::into_static),
        }
    }
}

impl FormatInto<Rust> for &Attr<'_> {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        if self.args.is_empty() {
            quote_in! { *tokens =>  #[$(&*self.name)]$['\r'] }
        } else {
            quote_in! { *tokens => #[$(&*self.name)($(for arg in self.args.iter() join (, ) => $arg))]$['\r'] }
        }
    }
}

impl FormatInto<Rust> for &AttrArg<'_> {
    fn format_into(self, tokens: &mut Tokens<Rust>) {
        match self {
            AttrArg {
                arg_name,
                arg_value: Some(value),
            } => {
                quote_in! { *tokens => $(&**arg_name) = $(&**value) }
            }
            AttrArg {
                arg_name,
                arg_value: None,
            } => {
                quote_in! { *tokens => $(&**arg_name) }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder)]
pub struct StructField<'a> {
    #[builder(setter(into))]
    pub name: Cow<'a, str>,
    #[builder(setter(into))]
    pub doc: Option<Cow<'a, str>>,
    #[builder(default)]
    pub attrs: Vec<Attr<'a>>,
    #[builder(setter(into))]
    pub ty: SharedOrOwned<TypeRef>,
}

impl<'a> StructField<'a> {
    pub fn has_lifetime(&self, ctx: &Context<'_>) -> Option<bool> {
        self.ty.has_lifetime(ctx)
    }

    pub fn check_for_boxing(&self, parent_name: &str, ctx: &Context<'_>) {
        if parent_name == "QueryParameterValue" && self.name.contains("range") {
            println!("{parent_name}.{}: {:#?}", self.name, self.ty);
        }

        let TypeRefKind::Generated(ref gener) = self.ty.kind else {
            return;
        };

        if self.has_recursive_type(parent_name, ctx) {
            gener.needs_boxing.set(true);
        }
    }

    pub fn has_recursive_type(&self, parent_name: &str, ctx: &Context<'_>) -> bool {
        if parent_name == "QueryParameterValue" && self.name.contains("range") {
            println!("{parent_name}.{}: {:#?}", self.name, self.ty);
        }

        let TypeRefKind::Generated(ref gener) = self.ty.kind else {
            return false;
        };

        let type_def = ctx.get_type_def(&gener.id);

        let type_def = type_def.borrow();

        if let TypeDefKind::Struct(ref fields) = type_def.kind {
            return fields
                .iter()
                .any(|field| field.has_recursive_type(parent_name, ctx));
        }

        false
    }
}

impl<'a> IntoStatic<'a> for StructField<'a> {
    type Static = StructField<'static>;

    fn into_static(self) -> Self::Static {
        StructField {
            name: self.name.into_static(),
            doc: self.doc.map(IntoStatic::into_static),
            attrs: self.attrs.into_static(),
            ty: self.ty,
        }
    }
}

impl GenerateCode<Rust> for StructField<'_> {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<Rust>) {
        let (name, rename_attr) = to_snake(&self.name);

        let default = self.ty.has_default(ctx).unwrap_or_default();

        quote_in! { *tokens =>
            $(ctx.doc_opt(self.doc.as_ref(), 0))
            $(for attr in self.attrs.iter() => $attr$['\r'])
            $(if let Some(field_attr) = &self.ty.field_attr => $field_attr$['\r'])
            $rename_attr$['\r']
            $(if default => #[serde(default)]$['\r'])
            pub $(&*name): $(self.ty.as_format_into(ctx)),$['\r']
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct EnumVariant<'a> {
    #[builder(setter(into))]
    pub name: Cow<'a, str>,
    #[builder(default)]
    pub attrs: Vec<Attr<'a>>,
    #[builder(setter(into, strip_option))]
    pub doc: Option<Cow<'a, str>>,
}

impl EnumVariant<'static> {
    pub(crate) fn from_name_and_doc(name: String, doc: String) -> Self {
        Self::builder().name(name).doc(doc).build()
    }
}

impl<'a> IntoStatic<'a> for EnumVariant<'a> {
    type Static = EnumVariant<'static>;

    fn into_static(self) -> Self::Static {
        EnumVariant {
            name: self.name.into_static(),
            attrs: self.attrs.into_static(),
            doc: self.doc.map(IntoStatic::into_static),
        }
    }
}

impl GenerateCode<Rust> for EnumVariant<'_> {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<Rust>) {
        let variant_name = to_pascal(self.name.as_ref());

        // ignore unspecified variants
        if variant_name.contains("Unspecified") {
            return;
        }

        quote_in! { *tokens =>
            $(ctx.doc_opt(self.doc.as_ref(), 1))
            $(for attr in self.attrs.iter() => $attr$['\r'])
            $(if self.name != variant_name {
                #[serde(rename = $(quoted(&*self.name)))]
            })
            $variant_name,$['\r']
        }
    }
}
