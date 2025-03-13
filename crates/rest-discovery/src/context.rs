use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::io::{self, Write};
use std::rc::Rc;

use genco::prelude::*;

use super::{GenerateCode, ir};
use crate::type_defs::{TypeCache, TypeId};

const STRING_INT_VISITORS_CODE: &str = include_str!("string_int_visitors.rs");

#[derive(Debug)]
pub struct Context<'a> {
    pub type_cache: Rc<RefCell<TypeCache<'a>>>,
    module_doc: Option<Cow<'a, str>>,
    base_url: Option<Cow<'a, str>>,
    config: super::CodeGenConfig,
    buffer: RefCell<String>,
}

impl<'a> Context<'a> {
    pub fn config(&self) -> &crate::CodeGenConfig {
        &self.config
    }

    pub(crate) fn new(config: super::CodeGenConfig) -> Self {
        Self {
            type_cache: Rc::new(RefCell::new(TypeCache::default())),
            module_doc: None,
            base_url: None,
            config,
            buffer: RefCell::new(String::new()),
        }
    }
    pub fn get_or_insert_type_def<F>(&self, name: Rc<str>, builder: F) -> ir::TypeRef
    where
        F: FnOnce(TypeId) -> ir::TypeDef<'a>,
    {
        self.type_cache
            .borrow_mut()
            .get_or_insert_type_def(self, name, builder)
    }

    pub fn find_type(&self, name: &str) -> Option<ir::TypeRef> {
        self.type_cache.borrow().find_type(name)
    }

    pub fn get_type_def(&self, id: &TypeId) -> Rc<RefCell<ir::TypeDef<'a>>> {
        self.type_cache.borrow().get_type_def(id)
    }

    pub fn resolve_types(&mut self) {
        /*
        let mut types = self.types.borrow_mut();

        for type_def in types.iter_mut() {
            let mut type_def = type_def.borrow_mut();

            if let TypeDefKind::Struct(ref mut fields) = type_def.kind {
                for field in fields {
                    todo!()
                }
            }
        }
        */
    }

    pub fn write_out<W: Write>(mut self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "// @generated")?;
        if let Some(doc) = self.module_doc.take() {
            crate::doc::DocFormatter::new_module_doc(&doc, 0, self.buffer.borrow_mut())
                .io_write_into(writer)?;
            writeln!(writer)?;
        }

        let mut tokens = genco::Tokens::new();

        if let Some(base_url) = self.base_url.take() {
            quote_in! { tokens =>
                /// The Base URL for this service.
                pub const BASE_URL: &str = $(quoted(base_url.trim()));
                $['\r']
                $['\n']
            }
        }

        self.type_cache.borrow().generate_code(&self, &mut tokens);

        let lang_cfg = rust::Config::default();
        let fmt_cfg = genco::fmt::Config::from_lang::<Rust>();

        let mut token_writer = genco::fmt::IoWriter::new(writer);
        tokens
            .format_file(&mut token_writer.as_formatter(&fmt_cfg), &lang_cfg)
            .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err))?;

        writeln!(token_writer.into_inner(), "\n{STRING_INT_VISITORS_CODE}")
    }

    pub(crate) fn get_derives<S: AsRef<str>>(&self, type_name: S) -> Option<&HashSet<String>> {
        self.config.extra_derive.get(type_name.as_ref())
    }

    pub fn add_base_url(&mut self, s: Cow<'a, str>) {
        self.base_url = Some(s);
    }
    pub fn add_module_doc(&mut self, s: Cow<'a, str>) {
        self.module_doc = match self.module_doc.take() {
            Some(existing) => {
                let mut ex = existing.into_owned();
                ex.push_str("\n\n");
                ex.push_str(&*s);
                Some(Cow::Owned(ex))
            }
            None => Some(s),
        }
    }

    pub fn doc<'b>(
        &'b self,
        doc_string: &'b str,
        indent_level: usize,
    ) -> crate::doc::DocFormatter<'b, RefMut<'b, String>> {
        crate::doc::DocFormatter::new_doc(doc_string, indent_level, self.buffer.borrow_mut())
    }

    pub fn doc_opt<'b, S>(
        &'b self,
        doc_string: Option<&'b S>,
        indent_level: usize,
    ) -> Option<crate::doc::DocFormatter<'b, RefMut<'b, String>>>
    where
        S: AsRef<str>,
    {
        doc_string.map(|s| self.doc(s.as_ref(), indent_level))
    }
}
