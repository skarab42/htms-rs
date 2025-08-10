#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../readme.md")]

use std::{fs::File, path::Path};

use darling::FromMeta;
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

    let template_attribute = attributes.template.value();
    let template_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&template_attribute);

    let template_file = match File::open(&template_path) {
        Err(error) => {
            return syn::Error::new_spanned(
                &attributes.template,
                format!(
                    "#[htms] failed to read template `{}`: {error}",
                    template_path.display()
                ),
            )
            .to_compile_error()
            .into();
        },
        Ok(item) => item,
    };

    println!("template_file: {template_file:?}");
    // todo: parse template file and get method names

    let trait_ident = format_ident!("{}HtmsTemplate", struct_ident);
    let method_idents = ["news", "blog_posts"]
        .into_iter()
        .map(|n| format_ident!("{}_task", n))
        .collect::<Vec<_>>();

    let method_signatures = method_idents.iter().map(|m| {
        quote! { fn #m(&self) -> impl ::core::future::Future<Output = ::std::string::String>; }
    });

    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();

    quote! {
        #original_item

        pub trait #trait_ident {
            #(#method_signatures)*
        }

        impl #impl_generics ::htms_template::Template for #struct_ident #ty_generics #where_clause {
             fn render(self) -> String {
                "Rendered template...".to_string()
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn htms(attributes: TokenStream, item: TokenStream) -> TokenStream {
    htms_impl(attributes, item)
}
