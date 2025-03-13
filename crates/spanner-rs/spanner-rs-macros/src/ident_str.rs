use std::rc::Rc;

use proc_macro2::Span;

/// invalid state if all 3 fields are None
#[derive(Debug)]
pub struct IdentStr {
    ident: Option<Rc<syn::Ident>>,
    string: Option<Rc<str>>,
    literal: Option<Rc<syn::LitStr>>,
}

macro_rules! ident {
    ($s:expr) => {{ Rc::new(syn::Ident::new(&$s, Span::call_site())) }};
}

macro_rules! litstr {
    ($s:expr) => {{ Rc::new(syn::LitStr::new(&$s, Span::call_site())) }};
}

impl IdentStr {
    pub fn ident(&mut self) -> Rc<syn::Ident> {
        Rc::clone(self.ident_ref())
    }

    pub fn ident_ref(&mut self) -> &Rc<syn::Ident> {
        match self.ident {
            Some(ref ident) => ident,
            None => match (&self.string, &self.literal) {
                (Some(s), _) => self.ident.insert(ident!(s)),
                (None, Some(lit)) => {
                    let s_refer = self.string.insert(Rc::from(lit.value()));
                    self.ident.insert(ident!(s_refer))
                }
                (None, None) => unreachable!(),
            },
        }
    }

    pub fn string(&mut self) -> Rc<str> {
        Rc::clone(self.string_ref())
    }

    pub fn string_ref(&mut self) -> &Rc<str> {
        match self.string {
            Some(ref s) => s,
            None => match (&self.literal, &self.ident) {
                (Some(s), _) => self.string.insert(Rc::from(s.value())),
                (None, Some(ident)) => self.string.insert(Rc::from(ident.to_string())),
                (None, None) => unreachable!(),
            },
        }
    }

    pub fn literal(&mut self) -> Rc<syn::LitStr> {
        Rc::clone(self.literal_ref())
    }

    pub fn literal_ref(&mut self) -> &Rc<syn::LitStr> {
        match self.literal {
            Some(ref s) => s,
            None => match (&self.string, &self.ident) {
                (Some(s), _) => self.literal.insert(litstr!(s)),
                (None, Some(ident)) => {
                    let s_refer = self.string.insert(Rc::from(ident.to_string()));
                    self.literal.insert(litstr!(s_refer))
                }
                (None, None) => unreachable!(),
            },
        }
    }
}

impl From<String> for IdentStr {
    fn from(value: String) -> Self {
        Self {
            string: Some(Rc::from(value)),
            ident: None,
            literal: None,
        }
    }
}

impl From<syn::Ident> for IdentStr {
    fn from(value: syn::Ident) -> Self {
        Self {
            ident: Some(Rc::new(value)),
            string: None,
            literal: None,
        }
    }
}

impl From<syn::LitStr> for IdentStr {
    fn from(value: syn::LitStr) -> Self {
        Self {
            ident: None,
            string: None,
            literal: Some(Rc::new(value)),
        }
    }
}
