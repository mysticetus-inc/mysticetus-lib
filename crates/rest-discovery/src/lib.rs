#![feature(type_changing_struct_update)]

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::path::Path;

// contained code is meant to be written out in the generated code,
// but might as well run typeck when testing.
#[cfg(test)]
mod string_int_visitors;

mod context;
mod doc;
mod impls;
mod ir;
mod type_defs;
mod types;

// mod test;

use context::Context;
use genco::prelude::*;
use ir::SharedOrOwned;

pub(crate) trait IntoStatic<'a>: 'a {
    type Static: 'static;

    fn into_static(self) -> Self::Static;
}

impl<'a, T> IntoStatic<'a> for Cow<'a, T>
where
    T: ToOwned + ?Sized + 'static,
{
    type Static = Cow<'static, T>;

    fn into_static(self) -> Self::Static {
        Cow::Owned(self.into_owned())
    }
}

impl<'a, T> IntoStatic<'a> for Vec<T>
where
    T: IntoStatic<'a>,
{
    type Static = Vec<T::Static>;

    fn into_static(self) -> Self::Static {
        self.into_iter().map(T::into_static).collect()
    }
}

pub trait GenerateIr<'a, L: Lang>: Sized {
    fn generate(self, ctx: &Context<'a>) -> anyhow::Result<SharedOrOwned<ir::TypeRef>>;
}

pub trait GenerateCode<L: Lang>: Sized {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<L>);

    fn as_format_into<'a, 'ctx>(
        &'a self,
        ctx: &'a Context<'ctx>,
    ) -> FormatIntoWrapper<'a, 'ctx, L, Self> {
        FormatIntoWrapper {
            item: self,
            ctx,
            _marker: PhantomData,
        }
    }
}

mod private {
    pub trait Sealed {}
}

impl<L, T> GenerateCode<L> for T
where
    T: FormatInto<L> + Clone + private::Sealed,
    L: Lang,
{
    fn generate_code(&self, _ctx: &Context<'_>, tokens: &mut Tokens<L>) {
        self.clone().format_into(tokens)
    }
}

impl<L, T> GenerateCode<L> for Vec<T>
where
    T: GenerateCode<L>,
    L: Lang,
{
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut Tokens<L>) {
        for item in self {
            item.generate_code(ctx, tokens);
        }
    }
}

pub struct FormatIntoWrapper<'a, 'ctx, L, T> {
    item: &'a T,
    ctx: &'a Context<'ctx>,
    _marker: PhantomData<L>,
}

impl<T, L> FormatInto<L> for FormatIntoWrapper<'_, '_, L, T>
where
    T: GenerateCode<L>,
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        self.item.generate_code(self.ctx, tokens);
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CodeGenConfig<P = (), U = ()> {
    use_cowstr: bool,
    use_btree_map: bool,
    use_bytes: bool,
    format_code: bool,
    extra_derive: HashMap<String, HashSet<String>>,
    override_optional: HashMap<String, HashSet<String>>,
    output_path: P,
    url: U,
}

impl CodeGenConfig<(), ()> {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl<U> CodeGenConfig<(), U> {
    pub fn output_file<P>(self, output_path: P) -> CodeGenConfig<P, U> {
        CodeGenConfig {
            output_path,
            ..self
        }
    }
}

impl<P> CodeGenConfig<P, ()> {
    pub fn discovery_url<U>(self, url: U) -> CodeGenConfig<P, U> {
        CodeGenConfig { url, ..self }
    }
}

impl<P, U> CodeGenConfig<P, U> {
    pub fn use_cowstr(mut self) -> Self {
        self.use_cowstr = true;
        self
    }

    pub fn use_btree_map(mut self) -> Self {
        self.use_btree_map = true;
        self
    }

    pub fn use_bytes(mut self) -> Self {
        self.use_bytes = true;
        self
    }

    pub fn format_generated_code(mut self) -> Self {
        self.format_code = true;
        self
    }

    pub fn add_optional_override<S, I>(mut self, type_name: S, fields: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<String>,
    {
        Self::add_to_map_inner(
            &mut self.override_optional,
            type_name.into(),
            fields.into_iter().map(Into::into),
        );
        self
    }

    pub fn add_optional_overrides<I, T, O>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (T, O)>,
        T: Into<String>,
        O: IntoIterator,
        O::Item: Into<String>,
    {
        for (type_name, field_iter) in iter {
            Self::add_to_map_inner(
                &mut self.override_optional,
                type_name.into(),
                field_iter.into_iter().map(Into::into),
            );
        }
        self
    }

    // internal helper fn for extending or inserting to a map
    fn add_to_map_inner<I>(map: &mut HashMap<String, HashSet<String>>, key: String, values: I)
    where
        I: Iterator<Item = String>,
    {
        match map.entry(key) {
            Entry::Occupied(existing) => {
                existing.into_mut().extend(values);
            }
            Entry::Vacant(vacant) => {
                vacant.insert(values.collect::<HashSet<String>>());
            }
        }
    }

    pub fn extend_extra_derive<I, T, D>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (T, D)>,
        T: Into<String>,
        D: IntoIterator,
        D::Item: Into<String>,
    {
        for (name, derives) in iter {
            Self::add_to_map_inner(
                &mut self.extra_derive,
                name.into(),
                derives.into_iter().map(Into::into),
            );
        }

        self
    }

    pub fn add_extra_derive<S, D>(mut self, type_name: S, derive: D) -> Self
    where
        S: Into<String>,
        D: IntoIterator,
        D::Item: Into<String>,
    {
        Self::add_to_map_inner(
            &mut self.extra_derive,
            type_name.into(),
            derive.into_iter().map(Into::into),
        );
        self
    }
}

impl<P, U> CodeGenConfig<P, U>
where
    P: AsRef<Path>,
    U: reqwest::IntoUrl,
{
    pub async fn generate(self) -> anyhow::Result<()> {
        let Self {
            use_cowstr,
            use_btree_map,
            use_bytes,
            format_code,
            extra_derive,
            override_optional,
            url,
            output_path,
        } = self;

        let config = CodeGenConfig {
            use_cowstr,
            use_btree_map,
            use_bytes,
            override_optional,
            format_code,
            extra_derive,
            ..Default::default()
        };
        generate(url, config, output_path.as_ref()).await
    }
}

async fn generate<U: reqwest::IntoUrl>(
    url: U,
    config: CodeGenConfig,
    output_file: &Path,
) -> anyhow::Result<()> {
    let doc = get_discovery_document(url).await?;

    let run_rustfmt = config.format_code;
    let mut ir_ctx = Context::new(config);

    doc.generate(&mut ir_ctx)?;

    let mut dst = std::fs::File::create(output_file)?;

    ir_ctx.write_out(&mut dst)?;

    if run_rustfmt {
        std::process::Command::new("rustfmt")
            .arg(output_file)
            .status()?;
    }

    Ok(())
}

async fn get_discovery_document(url: impl reqwest::IntoUrl) -> anyhow::Result<types::Discovery> {
    let string = reqwest::get(url).await?.error_for_status()?.text().await?;

    match serde_json::from_str(&string) {
        Ok(val) => Ok(val),
        Err(error) => {
            let (pos, _) = string
                .char_indices()
                .filter(|(_, c)| *c == '\n')
                .nth(error.line() - 1)
                .unwrap_or((0, ' '));

            let start = string.floor_char_boundary(pos.saturating_sub(500));
            let end = string.ceil_char_boundary(pos.saturating_add(100));

            let ctx = string.get(start..end).unwrap_or_default();

            Err(anyhow::anyhow!("{error}:\n'{ctx}'"))
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_gen() -> anyhow::Result<()> {
    const BQ_DISCOVERY: &str = "https://bigquery.googleapis.com/discovery/v1/apis/bigquery/v2/rest";

    CodeGenConfig::new()
        .output_file("./src/test.rs")
        .discovery_url(BQ_DISCOVERY)
        .generate()
        .await
}
