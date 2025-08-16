#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../readme.md")]

mod derive;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Template, attributes(template, context))]
pub fn template_derive(input: TokenStream) -> TokenStream {
    derive::template(&parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(derive::Error::into_compile_error)
}
