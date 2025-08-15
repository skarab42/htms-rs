#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../readme.md")]

mod htms;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn htms(attributes: TokenStream, item: TokenStream) -> TokenStream {
    htms::htms(attributes, item).unwrap_or_else(htms::Error::into_compile_error)
}
