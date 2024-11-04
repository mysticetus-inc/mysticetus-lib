const PROJECTS_PREFIX: &str = "projects/";
const INSTANCE_PREFIX: &str = "/instances/";
const DATABASE_PREFIX: &str = "/databases/";

pub(super) fn parse(tokens: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    // parse the parts
    let parts: Parts = syn::parse(tokens)?;

    // validate that all fields wont make google mad
    parts.project.validate("project", &[b'-'])?;
    parts.instance.validate("instance", &[b'-'])?;
    parts.database.validate("database", &[b'-', b'\\', b'_'])?;

    let mut buf = itoa::Buffer::new();

    // build all the indices so we can pick out each component from a single string
    // containing the entire qualified path
    let project_id_end = PROJECTS_PREFIX.len() + parts.project.string.len();
    let project_id_end_lit =
        syn::LitInt::new(buf.format(project_id_end), proc_macro2::Span::call_site());

    let instance_id_start = project_id_end + INSTANCE_PREFIX.len();
    let instance_id_start_lit = syn::LitInt::new(
        buf.format(instance_id_start),
        proc_macro2::Span::call_site(),
    );

    let instance_id_end = instance_id_start + parts.instance.string.len();
    let instance_id_end_lit =
        syn::LitInt::new(buf.format(instance_id_end), proc_macro2::Span::call_site());

    let database_start = instance_id_end + DATABASE_PREFIX.len();
    let database_start_lit =
        syn::LitInt::new(buf.format(database_start), proc_macro2::Span::call_site());

    // build the qualified path
    let mut qualified = String::with_capacity(
        PROJECTS_PREFIX.len()
            + parts.project.string.len()
            + INSTANCE_PREFIX.len()
            + parts.instance.string.len()
            + DATABASE_PREFIX.len()
            + parts.database.string.len(),
    );

    qualified.extend([
        PROJECTS_PREFIX,
        &parts.project.string,
        INSTANCE_PREFIX,
        &parts.instance.string,
        DATABASE_PREFIX,
        &parts.database.string,
    ]);

    let qualified_lit = syn::LitStr::new(&qualified, proc_macro2::Span::call_site());

    if let Some(crate_name) = parts.crate_name {
        Ok(quote::quote! {{
            #crate_name::info::Database::__from_parts(
                #qualified_lit,
                #project_id_end_lit,
                (#instance_id_start_lit, #instance_id_end_lit),
                #database_start_lit,
            )
        }})
    } else {
        Ok(quote::quote! {{
            ::spanner_rs::info::Database::__from_parts(
                #qualified_lit,
                #project_id_end_lit,
                (#instance_id_start_lit, #instance_id_end_lit),
                #database_start_lit,
            )
        }})
    }
}

#[derive(Debug)]
struct Parts {
    crate_name: Option<syn::Ident>,
    project: Part,
    instance: Part,
    database: Part,
}

#[derive(Debug)]
struct Part {
    lit: syn::LitStr,
    string: String,
}

impl Part {
    fn validate(&self, field_name: &str, allowed_middle_chars: &[u8]) -> syn::Result<()> {
        macro_rules! err {
            ($message:literal) => {{
                let mut msg = String::with_capacity(field_name.len() + 2 + $message.len());
                msg.push_str(field_name);
                msg.push_str(": ");
                msg.push_str($message);
                Err(syn::Error::new(self.lit.span(), msg))
            }};
        }

        if self.string.len() < 2 {
            return err!("literal too short (must be 2+ chracters)");
        }

        if !matches!(self.string.as_bytes()[0], b'a'..=b'z') {
            return err!("first character must start with a lowercase alphabetic letter");
        }

        let mut i = 1;
        while i < self.string.len() {
            let byte = self.string.as_bytes()[i];

            if !matches!(byte, b'0'..=b'9' | b'a'..=b'z') {
                if i == self.string.len() - 1 {
                    return err!("final character must be an ASCII alphanumeric character");
                }

                let mut allow_idx = 0;
                let mut found_allowed = false;
                while allow_idx < allowed_middle_chars.len() {
                    if byte == allowed_middle_chars[allow_idx] {
                        found_allowed = true;
                        break;
                    }
                    allow_idx += 1;
                }

                if !found_allowed {
                    return err!("found invalid character");
                }
            }

            i += 1;
        }

        Ok(())
    }
}

impl syn::parse::Parse for Parts {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        fn insert_or_error(
            dst: &mut Option<syn::LitStr>,
            src: syn::LitStr,
            field: &str,
        ) -> syn::Result<()> {
            if dst.is_some() {
                return Err(syn::Error::new(
                    src.span(),
                    format!("duplicate literal for field '{field}': {}", src.value()),
                ));
            }

            *dst = Some(src);
            Ok(())
        }

        let mut crate_name = None;

        let mut project: Option<syn::LitStr> = None;
        let mut instance: Option<syn::LitStr> = None;
        let mut database: Option<syn::LitStr> = None;

        let mut part_index = 0;
        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(syn::Ident) {
                let ident = input.parse::<syn::Ident>()?;

                input.parse::<syn::Token![:]>()?;

                if ident.eq("crate_name") {
                    let crate_name_ident = input.parse::<syn::Ident>()?;
                    crate_name = Some(crate_name_ident);
                    continue;
                }

                let lit = input.parse::<syn::LitStr>()?;

                if ident.eq("project") || ident.eq("project_id") {
                    insert_or_error(&mut project, lit, "project")?;
                } else if ident.eq("instance") || ident.eq("instance_id") {
                    insert_or_error(&mut instance, lit, "instance")?;
                } else if ident.eq("database") || ident.eq("database_id") {
                    insert_or_error(&mut database, lit, "database")?;
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown ident, expected one of: 'project', 'instance' or 'database' \
                             (with or without an '_id' suffix)"
                        ),
                    ));
                }
            } else if lookahead.peek(syn::LitStr) {
                let lit = input.parse::<syn::LitStr>()?;
                match part_index {
                    0 => insert_or_error(&mut project, lit, "project")?,
                    1 => insert_or_error(&mut instance, lit, "instance")?,
                    2 => insert_or_error(&mut database, lit, "database")?,
                    _ => {
                        return Err(syn::Error::new(
                            lit.span(),
                            "unknown literal meaning (cant tell what field it belongs to)",
                        ));
                    }
                }
            } else {
                return Err(lookahead.error());
            }

            if input.lookahead1().peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }

            part_index += 1;
        }

        macro_rules! build_part_or_err {
            ($item:expr) => {{
                match $item {
                    Some(lit) => Part {
                        string: lit.value(),
                        lit,
                    },
                    None => {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            concat!("required ", stringify!($item), " value wasn't provided"),
                        ));
                    }
                }
            }};
        }

        let project = build_part_or_err!(project);
        let instance = build_part_or_err!(instance);
        let database = build_part_or_err!(database);

        Ok(Self {
            crate_name,
            project,
            instance,
            database,
        })
    }
}
