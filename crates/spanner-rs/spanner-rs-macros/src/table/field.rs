use std::rc::Rc;

use convert_case::{Case, Casing};

use crate::ident_str::IdentStr;

#[derive(Debug)]
pub enum FieldOpts {
    Pk { pk: PkIndex, base: BaseFieldOpts },
    Base(BaseFieldOpts),
}

impl FieldOpts {
    pub fn as_pk_mut(&mut self) -> Option<(&PkIndex, &mut BaseFieldOpts)> {
        match self {
            Self::Pk { pk, base } => Some((pk, base)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct BaseFieldOpts {
    pub base: IdentStr,
    pub pascal: IdentStr,
    pub screaming_snake: IdentStr,
    pub ty: syn::Type,
    pub is_option: bool,
    pub rename: Option<syn::LitStr>,
    pub marker_type: Option<Rc<syn::Ident>>,
    pub with: Option<syn::Ident>,
    pub with_serde_as: Option<syn::Type>,
}

impl BaseFieldOpts {
    pub fn col_marker_ident(&mut self) -> Rc<syn::Ident> {
        self.marker_type
            .clone()
            .unwrap_or_else(|| self.pascal.ident())
    }
}

#[derive(Debug)]
pub struct PkIndex {
    pub lit: syn::LitInt,
    pub index: usize,
}

impl PkIndex {
    fn new(lit: syn::LitInt) -> syn::Result<Self> {
        let index = lit.base10_parse()?;

        Ok(Self { lit, index })
    }
}

impl darling::FromField for FieldOpts {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        #[derive(FromField)]
        #[darling(attributes(spanner))]
        struct RawFieldOpts {
            ident: Option<syn::Ident>,
            ty: syn::Type,
            #[darling(default)]
            pk: Option<syn::LitInt>,
            #[darling(default)]
            rename: Option<syn::LitStr>,
            #[darling(default)]
            marker_type: Option<syn::Ident>,
            #[darling(default)]
            with: Option<syn::Ident>,
            #[darling(default)]
            with_serde_as: Option<syn::Type>,
            #[darling(default)]
            nullable: bool,
        }

        let RawFieldOpts {
            ident,
            ty,
            pk,
            rename,
            marker_type,
            with,
            with_serde_as,
            nullable,
        } = RawFieldOpts::from_field(field)?;

        let ident = ident.ok_or_else(|| darling::Error::unsupported_shape("Tuple"))?;

        let mut base = IdentStr::from(ident);
        let base_str = base.string();

        if with.is_some() && with_serde_as.is_some() {
            return Err(darling::Error::custom(
                "only one of 'with' and 'with_serde_as' are supported, not both",
            ));
        }
        let base = BaseFieldOpts {
            pascal: IdentStr::from(base_str.as_ref().to_case(Case::Pascal)),
            screaming_snake: IdentStr::from(base_str.as_ref().to_case(Case::ScreamingSnake)),
            base,
            is_option: nullable || check_for_option(&ty),
            ty,
            rename,
            marker_type: marker_type.map(Rc::new),
            with,
            with_serde_as,
        };

        match pk.map(PkIndex::new).transpose()? {
            Some(pk) => Ok(FieldOpts::Pk { pk, base }),
            None => Ok(FieldOpts::Base(base)),
        }
    }
}

fn check_for_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(path) = ty
        && let Some(last) = path.path.segments.last()
    {
        last.ident.eq("Option")
    } else {
        false
    }
}

impl std::ops::Deref for FieldOpts {
    type Target = BaseFieldOpts;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Pk { base, .. } | Self::Base(base) => base,
        }
    }
}

impl std::ops::DerefMut for FieldOpts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Pk { base, .. } | Self::Base(base) => base,
        }
    }
}

// these 2 macros rely on deref giving psuedo-field access to 'ty'
uses_type_params!(FieldOpts, ty);
uses_lifetimes!(FieldOpts, ty);
