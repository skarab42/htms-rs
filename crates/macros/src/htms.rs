use std::{env, path::PathBuf, result};

use darling::FromMeta;
use htms_core::template;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{ItemStruct, LitStr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Syn(#[from] syn::Error),
    #[error(transparent)]
    Template(#[from] template::Error),
    #[error("failed to get env var `{0}`: {1}")]
    EnvVar(&'static str, #[source] env::VarError),
}

impl Error {
    pub fn into_compile_error(self) -> TokenStream {
        match self {
            Self::Syn(error) => error.to_compile_error().into(),
            error => {
                let message = Literal::string(error.to_string().as_str());

                quote!(compile_error!(#message)).into()
            },
        }
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct HtmsAttributes {
    template: LitStr,
}

fn try_env_var(name: &'static str) -> Result<String> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(error) => Err(Error::EnvVar(name, error)),
    }
}

pub fn htms(attributes: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attributes: HtmsAttributes = syn::parse(attributes)?;
    let original_item_struct = proc_macro2::TokenStream::from(item.clone());
    let item_struct: ItemStruct = syn::parse(item)?;

    // TODO: add custom env var for template root directory ?
    let manifest_path = PathBuf::from(&try_env_var("CARGO_MANIFEST_DIR")?);
    let template_attribute = attributes.template.value();
    let input_template_path = manifest_path.join(&template_attribute);
    let build_dir = manifest_path.join(".htms").join("output");
    let output_template_path = build_dir.join(&template_attribute);
    let template_include_str = Literal::string(&output_template_path.to_string_lossy());

    // TODO: implement some sort of cache?
    let build = template::parse_and_build(input_template_path, output_template_path)?;

    let item_struct_ident = format_ident!("{}", item_struct.ident);
    let item_trait_ident = format_ident!("{}Render", item_struct.ident);

    let task_names_str = build
        .task_names()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    let method_idents = build
        .task_names()
        .iter()
        .map(|name| format_ident!("{}_task", name))
        .collect::<Vec<_>>();

    let task_signatures = method_idents.iter().map(|task_signature| {
        quote! { fn #task_signature() -> impl ::core::future::Future<Output = ::std::string::String> + Send; }
    });

    let final_chunk_tokens = if build.has_html_tag() {
        quote! { Some(::htms_core::Bytes::from_static(b"</body></html>")) }
    } else {
        quote! { None }
    };

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    let code = quote! {
        #original_item_struct

        pub trait #item_trait_ident {
            #(#task_signatures)*
        }

        impl #impl_generics ::htms_core::Render for #item_struct_ident #ty_generics #where_clause {
            fn tasks() -> Option<Vec<::htms_core::Task>> {
                Some(vec![#(::htms_core::Task::new(#task_names_str, Self::#method_idents()),)*])
            }

            fn template() -> ::htms_core::Bytes {
                ::htms_core::Bytes::from_static(include_bytes!(#template_include_str))
            }

            fn final_chunk() -> Option<::htms_core::Bytes> {
                #final_chunk_tokens
            }
        }
    }
    .into();

    Ok(code)
}
