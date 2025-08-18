use std::{env, env::VarError, path::PathBuf, result};

use darling::{FromDeriveInput, FromField, ast::Data};
use htms_core::template;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, DeriveInput, Expr, ExprLit, Ident, ItemStruct, Lit, LitStr, Type};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Var(#[from] VarError),
    #[error(transparent)]
    Syn(#[from] syn::Error),
    #[error(transparent)]
    Darling(#[from] darling::Error),
}

impl Error {
    pub fn into_compile_error(self) -> TokenStream {
        match self {
            Self::Syn(err) => err.to_compile_error().into(),
            Self::Darling(err) => err.write_errors().into(),
            Self::Var(error) => {
                let message = format!("[htms] {error}");
                quote!(compile_error!(#message)).into()
            },
        }
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, FromField)]
#[darling(forward_attrs(context))]
struct TemplateField {
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<Attribute>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(context), forward_attrs(template), supports(struct_named))]
struct TemplateInput {
    ident: Ident,
    data: Data<(), TemplateField>,
    attrs: Vec<Attribute>,
}

pub fn template(input: &DeriveInput) -> Result<TokenStream> {
    let template_input = TemplateInput::from_derive_input(input)?;
    let template_path_lit = get_template_path_lit(&template_input)?;
    let context_field = find_context_field(&template_input)?;

    // TODO: allow to override the build path by env var
    let crate_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let build_path = crate_path.join(".htms").join("build");

    let template_path_input = crate_path.join(template_path_lit.value());
    let template_path_output = build_path.join(template_path_lit.value());
    let template_path_output_lit = Literal::string(&template_path_output.to_string_lossy());

    let build = template::parse_and_build(&template_path_input, &template_path_output)
        .map_err(|error| Error::Syn(syn::Error::new_spanned(&template_path_lit, error)))?;

    let input_struct: ItemStruct = syn::parse(input.to_token_stream().into())?;
    let (impl_generics, ty_generics, where_clause) = input_struct.generics.split_for_impl();

    let input_struct_ident = format_ident!("{}", input_struct.ident);
    let input_trait_ident = format_ident!("{}Render", input_struct.ident);

    let (task_names, method_idents) = build
        .task_names()
        .iter()
        .map(|name| (name.as_str(), format_ident!("{}_task", name)))
        .collect::<(Vec<_>, Vec<_>)>();

    let (task_arguments, context_field) = match context_field {
        Some(context) => {
            let ty = context.ty;
            let ident = context.ident;
            (quote! { context: #ty }, quote! { self.#ident.clone() })
        },
        None => (quote! {}, quote! {}),
    };

    let base_trait = quote! {
        pub trait #input_trait_ident {
            #(fn #method_idents(#task_arguments) -> impl ::core::future::Future<Output = ::std::string::String> + Send + 'static;)*
        }
    };

    let final_chunk_body = if build.has_html_tag() {
        quote! { Some(::htms_core::Bytes::from_static(b"</body></html>")) }
    } else {
        quote! { None }
    };

    let render_impl = quote! {
         use ::htms_core::Render;

         impl #impl_generics ::htms_core::Render for #input_struct_ident #ty_generics #where_clause {
            fn tasks(self) -> Option<Vec<::htms_core::Task>> {
                Some(vec![#(::htms_core::Task::new(#task_names, Self::#method_idents(#context_field)),)*])
            }

            fn template() -> ::htms_core::Bytes {
                ::htms_core::Bytes::from_static(include_bytes!(#template_path_output_lit))
            }

            fn final_chunk() -> Option<::htms_core::Bytes> {
                #final_chunk_body
            }
        }
    };

    Ok(quote! {
        #base_trait
        #render_impl
    }
    .into())
}

#[derive(Debug)]
struct ContextField {
    ident: Ident,
    ty: Type,
}

fn find_context_field(input: &TemplateInput) -> Result<Option<ContextField>> {
    let fields = get_template_fields(&input.data);

    let mut tagged = fields
        .iter()
        .filter(|field| has_attribute(&field.attrs, "context"));

    let Some(first) = tagged.next() else {
        return Ok(find_field(fields, "context").map(|field| ContextField {
            ident: field
                .ident
                .clone()
                .unwrap_or_else(|| unreachable!("supports(struct_named) ensures a struct")),
            ty: field.ty.clone(),
        }));
    };

    if let Some(second) = tagged.next() {
        let first_ident = first
            .ident
            .as_ref()
            .unwrap_or_else(|| unreachable!("supports(struct_named) ensures a struct"));
        let second_ident = second
            .ident
            .as_ref()
            .unwrap_or_else(|| unreachable!("supports(struct_named) ensures a struct"));

        let mut error = syn::Error::new_spanned(&input.ident, "duplicate #[context] attributes");

        error.combine(syn::Error::new_spanned(
            second_ident,
            "second #[context] attribute here",
        ));

        error.combine(syn::Error::new_spanned(
            first_ident,
            "first #[context] attribute is here",
        ));

        return Err(Error::Syn(error));
    }

    Ok(first.ident.clone().map(|ident| ContextField {
        ident,
        ty: first.ty.clone(),
    }))
}

fn find_field<I: AsRef<str>>(fields: &[TemplateField], ident: I) -> Option<&TemplateField> {
    fields
        .iter()
        .find(|field| field.ident.clone().is_some_and(|i| i.eq(&ident)))
}

fn find_attribute<I: AsRef<str>>(attributes: &[Attribute], ident: I) -> Option<&Attribute> {
    attributes
        .iter()
        .find(|a| a.path().is_ident(ident.as_ref()))
}

fn has_attribute<I: AsRef<str>>(attributes: &[Attribute], ident: I) -> bool {
    attributes.iter().any(|a| a.path().is_ident(ident.as_ref()))
}

fn get_template_fields(data: &Data<(), TemplateField>) -> &[TemplateField] {
    match data {
        Data::Struct(fields) => &fields.fields,
        Data::Enum(_) => unreachable!("supports(struct_named) ensures a struct"),
    }
}

fn get_template_path_lit(template_input: &TemplateInput) -> Result<LitStr> {
    let Some(template_attribute) = find_attribute(&template_input.attrs, "template") else {
        return Err(Error::Syn(syn::Error::new_spanned(
            &template_input.ident,
            r#"missing #[template = "..."] annotation"#,
        )));
    };

    match template_attribute.meta.require_name_value() {
        Ok(name_value) => match &name_value.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => Ok(s.clone()),
            expr => Err(Error::Syn(syn::Error::new_spanned(
                expr,
                "expected a string literal, eg. \"index.htm\"",
            ))),
        },
        Err(source) => {
            let mut error = syn::Error::new_spanned(
                &template_input.ident,
                format!(r#"invalid #[template = "..."] annotation: {source}"#),
            );

            error.combine(source);

            Err(Error::Syn(error))
        },
    }
}
