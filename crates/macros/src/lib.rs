#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../readme.md")]

use std::{env, path::PathBuf};

use darling::FromMeta;
use htms_core::template;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Item, ItemStruct, LitStr};

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct HtmsAttributes {
    template: LitStr,
}

fn htms_impl(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes: HtmsAttributes = match syn::parse(attributes) {
        Err(error) => return error.to_compile_error().into(),
        Ok(attributes) => attributes,
    };

    let original_item = proc_macro2::TokenStream::from(item.clone());
    let item: Item = match syn::parse(item) {
        Err(error) => return error.to_compile_error().into(),
        Ok(item) => item,
    };

    let Item::Struct(ItemStruct {
        ident: struct_ident,
        generics: struct_generics,
        ..
    }) = item
    else {
        return quote! { compile_error!("#[htms] must be applied to a `struct`"); }.into();
    };

    // TODO: add custom env var for template root directory ?
    let manifest_dir = env::var("CARGO_MANIFEST_DIR");
    let manifest_path = match manifest_dir {
        Ok(dir) => PathBuf::from(&dir),
        Err(error) => {
            return syn::Error::new_spanned(
                &attributes.template,
                format!("#[htms] failed to get manifest directory: {error}"),
            )
            .to_compile_error()
            .into();
        },
    };

    let template_attribute = attributes.template.value();
    let input_template_path = manifest_path.join(&template_attribute);
    let build_dir = manifest_path.join(".htms").join("output");
    let output_template_path = build_dir.join(&template_attribute);

    let template_include_str =
        LitStr::new(&output_template_path.to_string_lossy(), struct_ident.span());

    // TODO: implement some sort of cache
    let build = match template::parse_and_build(input_template_path, output_template_path) {
        Ok(task_name) => task_name,
        Err(error) => {
            return syn::Error::new_spanned(
                &attributes.template,
                format!("#[htms] failed to parse template: {error}"),
            )
            .to_compile_error()
            .into();
        },
    };

    let trait_ident = format_ident!("{}Render", struct_ident);
    let task_names_str = build
        .task_names()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let method_idents = task_names_str
        .iter()
        .map(|n| format_ident!("{}_task", n))
        .collect::<Vec<_>>();

    let method_signatures = method_idents.iter().map(|m| {
        quote! { fn #m() -> impl ::core::future::Future<Output = ::std::string::String> + Send; }
    });

    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();

    let final_chunk_tokens = if build.has_html_tag() {
        quote! { Some(::htms_core::Bytes::from_static(b"</body></html>")) }
    } else {
        quote! { None }
    };

    quote! {
        #original_item

        pub trait #trait_ident {
            #(#method_signatures)*
        }

        impl #impl_generics ::htms_core::Render for #struct_ident #ty_generics #where_clause {
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
    .into()
}

#[proc_macro_attribute]
pub fn htms(attributes: TokenStream, item: TokenStream) -> TokenStream {
    htms_impl(attributes, item)
}
