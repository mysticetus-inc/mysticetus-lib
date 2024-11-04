#![feature(proc_macro_diagnostic)]
use proc_macro::{Diagnostic, TokenStream};
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

enum HelperAttrType {
    Id,
    DisplayProps,
    Name,
    MiscProps,
    Timestamp,
    Client,
    Required,
}

impl HelperAttrType {
    fn new(attr: &syn::Attribute) -> Option<syn::Result<Self>> {
        if !attr.path.segments.iter().any(|seg| seg.ident.eq("geojson")) {
            None
        } else {
            Some(attr.parse_args())
        }
    }
}

impl syn::parse::Parse for HelperAttrType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let diag_val: syn::Ident = input.parse()?;

        if diag_val.eq("id") {
            Ok(Self::Id)
        } else if diag_val.eq("display_properties") {
            Ok(Self::DisplayProps)
        } else if diag_val.eq("name") {
            Ok(Self::Name)
        } else if diag_val.eq("misc") {
            Ok(Self::MiscProps)
        } else if diag_val.eq("client") {
            Ok(Self::Client)
        } else if diag_val.eq("required") {
            Ok(Self::Required)
        } else if diag_val.eq("time") || diag_val.eq("timestamp") || diag_val.eq("epoch") {
            Ok(Self::Timestamp)
        } else {
            Err(syn::Error::new(
                input.span(),
                format!("unknown geojson attribute type: '{}'", diag_val),
            ))
        }
    }
}

#[allow(dead_code)]
struct Struct {
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: syn::DataStruct,
}

impl TryFrom<DeriveInput> for Struct {
    type Error = &'static str;

    fn try_from(input: DeriveInput) -> Result<Self, Self::Error> {
        match input.data {
            syn::Data::Struct(data) => Ok(Struct {
                attrs: input.attrs,
                generics: input.generics,
                ident: input.ident,
                vis: input.vis,
                data,
            }),
            syn::Data::Enum(_) | syn::Data::Union(_) => {
                Err("GeoJsonProps can only be derived on structs")
            }
        }
    }
}

#[proc_macro_derive(GeoJsonProps, attributes(geojson))]
pub fn geojson_props_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_span = input.ident.span();

    match Struct::try_from(input) {
        Ok(input) => derive_impls(input),
        Err(string) => {
            let literal = syn::LitStr::new(string, input_span);
            TokenStream::from(quote! { ::core::compile_error!( #literal ) })
        }
    }
}

fn field_ident_eq<const N: usize>(field: &syn::Field, possible: [&str; N]) -> bool {
    let ident = match &field.ident {
        Some(ident) => ident,
        None => return false,
    };

    possible.iter().any(|choice| ident.eq(choice))
}

struct PropFields {
    id: Option<syn::Field>,
    name: Option<syn::Field>,
    misc: Option<syn::Field>,
    timestamp: Option<syn::Field>,
    display_props: Option<syn::Field>,
    client: Option<syn::Field>,
    required_fields: Vec<syn::Field>,
}

fn diagnostic_err(err: syn::Error) -> TokenStream {
    Diagnostic::spanned(
        proc_macro::Span::call_site(),
        proc_macro::Level::Error,
        err.to_string(),
    )
    .emit();

    TokenStream::new()
}

fn derive_impls(struct_info: Struct) -> TokenStream {
    let mut prop_fields = PropFields {
        id: None,
        name: None,
        misc: None,
        timestamp: None,
        client: None,
        display_props: None,
        required_fields: Vec::new(),
    };

    for field in struct_info.data.fields.iter() {
        for attr_result in field.attrs.iter().filter_map(HelperAttrType::new) {
            let attr_helper = match attr_result {
                Ok(attr) => attr,
                Err(error) => return diagnostic_err(error),
            };

            match attr_helper {
                HelperAttrType::Id if prop_fields.id.is_none() => {
                    prop_fields.id = Some(field.clone());
                    prop_fields.required_fields.push(field.clone());
                }
                HelperAttrType::DisplayProps if prop_fields.display_props.is_none() => {
                    prop_fields.display_props = Some(field.clone());
                }
                HelperAttrType::Name if prop_fields.name.is_none() => {
                    prop_fields.name = Some(field.clone());
                    prop_fields.required_fields.push(field.clone());
                }
                HelperAttrType::MiscProps if prop_fields.misc.is_none() => {
                    prop_fields.misc = Some(field.clone());
                }
                HelperAttrType::Timestamp if prop_fields.timestamp.is_none() => {
                    prop_fields.timestamp = Some(field.clone());
                    prop_fields.required_fields.push(field.clone());
                }
                HelperAttrType::Client if prop_fields.client.is_none() => {
                    prop_fields.client = Some(field.clone());
                    prop_fields.required_fields.push(field.clone());
                }
                HelperAttrType::Required => prop_fields.required_fields.push(field.clone()),
                _ => (),
            }
        }
    }

    for field in struct_info.data.fields.iter() {
        if prop_fields.display_props.is_none()
            && field_ident_eq(field, ["display_props", "display_properties"])
        {
            prop_fields.display_props = Some(field.clone());
        } else if prop_fields.id.is_none() && field_ident_eq(field, ["id"]) {
            prop_fields.id = Some(field.clone());
            prop_fields.required_fields.push(field.clone());
        } else if prop_fields.name.is_none() && field_ident_eq(field, ["name"]) {
            prop_fields.name = Some(field.clone());
            prop_fields.required_fields.push(field.clone());
        } else if prop_fields.misc.is_none() && field_ident_eq(field, ["misc", "props"]) {
            prop_fields.misc = Some(field.clone());
        } else if prop_fields.misc.is_none() && field_ident_eq(field, ["client"]) {
            prop_fields.client = Some(field.clone());
            prop_fields.required_fields.push(field.clone());
        } else if prop_fields.timestamp.is_none()
            && field_ident_eq(field, ["timestamp", "epoch", "time"])
        {
            prop_fields.timestamp = Some(field.clone());
            prop_fields.required_fields.push(field.clone());
        }
    }

    prop_fields.required_fields.sort_by_cached_key(|req_field| {
        struct_info
            .data
            .fields
            .iter()
            .position(|field| field.ident == req_field.ident)
    });

    let mut output_tokens = TokenStream::new();

    match derive_inner_prop_map(&struct_info, &prop_fields) {
        Ok(ok) => output_tokens.extend(ok),
        Err(err) => return diagnostic_err(err),
    }

    match derive_props(&struct_info, &prop_fields) {
        Ok(ok) => output_tokens.extend(ok),
        Err(err) => return diagnostic_err(err),
    }

    if let Some(disp_impl) = prop_fields
        .display_props
        .as_ref()
        .map(|field| derive_display_props(&struct_info, field))
    {
        output_tokens.extend(disp_impl);
    }

    output_tokens.extend(derive_timed_props(&struct_info, &prop_fields));

    output_tokens
}

fn derive_inner_prop_map(struc: &Struct, fields: &PropFields) -> syn::Result<TokenStream> {
    let struct_ident = &struc.ident;
    let (impl_generics, ty_generics, where_clause) = struc.generics.split_for_impl();

    let misc_field = fields.misc.as_ref().ok_or_else(|| {
        syn::Error::new(
            struc.ident.span(),
            "no 'misc' field found (or specified with 'geojson(misc)')",
        )
    })?;

    let misc_ty = &misc_field.ty;
    let misc_ident = &misc_field.ident;

    let quoted = quote! {
        impl #impl_generics ::geojson::properties::InnerPropertyMap
        for #struct_ident #ty_generics #where_clause {
            type Map = #misc_ty;

            fn into_property_map(self) -> Self::Map {
                self.#misc_ident
            }

            fn property_map(&self) -> &Self::Map {
                &self.#misc_ident
            }

            fn property_map_mut(&mut self) -> &mut Self::Map {
                &mut self.#misc_ident
            }
        }
    };

    Ok(TokenStream::from(quoted))
}

fn derive_props(struc: &Struct, fields: &PropFields) -> syn::Result<TokenStream> {
    let struct_ident = &struc.ident;
    let (impl_generics, ty_generics, where_clause) = struc.generics.split_for_impl();

    let name_field = fields.name.as_ref().ok_or_else(|| {
        syn::Error::new(
            struc.ident.span(),
            "no 'name' field found (or specified with 'geojson(name)')",
        )
    })?;

    let id_field = fields.id.as_ref().ok_or_else(|| {
        syn::Error::new(
            struc.ident.span(),
            "no 'id' field found (or specified with 'geojson(id)')",
        )
    })?;

    let id_ident = &id_field.ident;
    let id_type = &id_field.ty;
    let name_ident = &name_field.ident;
    let name_type = &name_field.ty;

    let required_pairs = fields.required_fields.iter().map(|field| {
        let ident = &field.ident;
        quote!( #ident: args.#ident, )
    });

    let prop_arg_fields = fields.required_fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        quote!( pub #ident: #ty, )
    });

    let remaining_fields = struc
        .data
        .fields
        .iter()
        .filter(|field| {
            !fields
                .required_fields
                .iter()
                .any(|req| req.ident == field.ident)
        })
        .map(|field| {
            let ident = &field.ident;
            quote!( #ident: ::std::default::Default::default(), )
        });

    let (name_ref_type, name_expr) = match &name_field.ty {
        syn::Type::Path(syn::TypePath { path, .. })
            if path
                .segments
                .last()
                .map(|seg| seg.ident.eq("String"))
                .unwrap_or_default() =>
        {
            let name_expr: proc_macro2::TokenStream =
                syn::parse_str("type NameRef<'a> = &'a str where Self: 'a;")?;
            (Some(name_expr), quote!( self.#name_ident.as_str() ))
        }
        _ => (None, quote!( &self.#name_ident )),
    };

    let mut prop_args_name = struct_ident.to_string();
    if prop_args_name.ends_with('s') {
        prop_args_name.pop();
    }
    prop_args_name.push_str("Args");

    let prop_args_ident = syn::Ident::new(prop_args_name.as_str(), proc_macro2::Span::call_site());

    let prop_args_type: syn::Type = syn::parse_str(prop_args_name.as_str())?;

    let quoted = quote! {
        pub struct #prop_args_ident {
            #( #prop_arg_fields )*
        }

        impl #impl_generics ::geojson::Properties
        for #struct_ident #ty_generics #where_clause {

            type Id = #id_type;

            type Name = #name_type;

            #name_ref_type

            type RequiredArgs = #prop_args_type;

            fn new(args: Self::RequiredArgs) -> Self {
                Self {
                    #( #required_pairs )*
                    #( #remaining_fields )*
                }
            }

            fn id(&self) -> Self::Id {
                self.#id_ident
            }

            fn set_id(&mut self, id: Self::Id) {
                self.#id_ident = id;
            }

            fn name(&self) -> Self::NameRef<'_> {
                #name_expr
            }

            fn name_mut(&mut self) -> &mut Self::Name {
                &mut self.#name_ident
            }
        }
    };

    /*
    #[cfg(feature = "inherent_associated_types")]
    {
        let quoted = quote! {
            #quoted
            impl #impl_generics #struct_ident #ty_generics #where_clause {
                type Args = #prop_args_type;
            }
        };
    }
    */

    Ok(TokenStream::from(quoted))
}

fn derive_display_props(struc: &Struct, field: &syn::Field) -> TokenStream {
    let struct_ident = &struc.ident;
    let (impl_generics, ty_generics, where_clause) = struc.generics.split_for_impl();

    let field_ident = &field.ident;
    let field_ty = &field.ty;

    let quoted = quote! {
        impl #impl_generics ::geojson::properties::DisplayProps
        for #struct_ident #ty_generics #where_clause {
            type DisplayProps = #field_ty;

            fn display_props(&self) -> &Self::DisplayProps {
                &self.#field_ident
            }

            fn display_props_mut(&mut self) -> &mut Self::DisplayProps {
                &mut self.#field_ident
            }
        }
    };

    TokenStream::from(quoted)
}

fn is_field_optional(field: &syn::Field) -> bool {
    match &field.ty {
        syn::Type::Path(syn::TypePath { path, .. }) => match path.segments.first() {
            Some(first) => first.ident.eq("Option"),
            _ => false,
        },
        _ => false,
    }
}

fn derive_timed_props(struc: &Struct, fields: &PropFields) -> TokenStream {
    let ts_field = match &fields.timestamp {
        Some(ts_field) if !is_field_optional(ts_field) => ts_field,
        _ => return TokenStream::new(),
    };

    let struct_ident = &struc.ident;
    let (impl_generics, ty_generics, where_clause) = struc.generics.split_for_impl();

    let field_ident = &ts_field.ident;

    let quoted = quote! {
        impl #impl_generics ::geojson::properties::TimedProps
        for #struct_ident #ty_generics #where_clause {
            fn timestamp(&self) -> ::timestamp::Timestamp {
                self.#field_ident
            }

            fn set_timestamp(&mut self, timestamp: ::timestamp::Timestamp) {
                self.#field_ident = timestamp;
            }
        }
    };

    TokenStream::from(quoted)
}
