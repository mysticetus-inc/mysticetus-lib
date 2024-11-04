use std::cell::RefCell;
use std::rc::Rc;

use genco::lang::Rust;
use indexmap::IndexMap;

use crate::context::Context;
use crate::{GenerateCode, ir};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId {
    index: usize,
    pub name: Rc<str>,
}

#[derive(Debug, Default)]
pub struct TypeCache<'a> {
    types: Vec<(Rc<RefCell<ir::TypeDef<'a>>>, ir::TypeRef)>,
    type_index: IndexMap<Rc<str>, TypeId>,
}

impl GenerateCode<Rust> for TypeCache<'_> {
    fn generate_code(&self, ctx: &Context<'_>, tokens: &mut genco::prelude::Tokens<Rust>) {
        for (type_def, _) in self.types.iter() {
            type_def.borrow().generate_code(ctx, tokens);
        }
    }
}

impl<'a> TypeCache<'a> {
    pub fn find_type(&self, name: impl AsRef<str>) -> Option<ir::TypeRef> {
        let id = self.type_index.get(name.as_ref())?;
        Some(self.get_type_ref(id))
    }

    pub fn get_type_ref(&self, id: &TypeId) -> ir::TypeRef {
        self.types[id.index].1.clone()
    }

    pub fn get_type_def(&self, id: &TypeId) -> Rc<RefCell<ir::TypeDef<'a>>> {
        self.types[id.index].0.clone()
    }

    pub fn get_or_insert_type_def<F>(
        &mut self,
        ctx: &Context<'a>,
        name: Rc<str>,
        builder: F,
    ) -> ir::TypeRef
    where
        F: FnOnce(TypeId) -> ir::TypeDef<'a>,
    {
        match self.type_index.get(&name).cloned() {
            Some(id) => self.get_type_ref(&id),
            None => {
                let id = TypeId {
                    index: self.types.len(),
                    name,
                };

                self.type_index.insert(id.name.clone(), id.clone());

                let new = builder(id.clone());
                let refer = new.as_type_ref(ctx);
                self.types.push((Rc::new(RefCell::new(new)), refer.clone()));
                refer
            }
        }
    }
}
