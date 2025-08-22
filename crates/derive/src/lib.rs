#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../readme.md")]

mod derive;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive `Template` for a struct to make it renderable with `htms`.
///
/// The macro wires your type into the template pipeline:
/// - Implements [`htms_core::Render`] for the struct.
/// - Generates a companion trait `<TypeName>Render` with hooks for async tasks
///   and an optional final chunk.
///
/// # Attributes
/// - `#[template = "path/to/file.html"]` (required)
///   Path to the HTML template, resolved relative to `CARGO_MANIFEST_DIR`.
/// - `#[context]` (optional)
///   Marks the field used as *context*. If not provided, a field named
///   `context` is used. The context type **must be `Clone`**.
///   If both are present, `#[context]` takes precedence.
///
/// # Generated items
/// - `impl htms_core::Render for YourType`
/// - `pub trait YourTypeRender { /* default hooks for tasks/final_chunk */ }`
///   Implement this trait for your type to provide async tasks or a final chunk.
///
/// # Example: template
/// ```html
#[doc = include_str!("../fixtures/example.html")]
/// ```
///
/// # Example: with tasks
///
/// ```rust
/// #[derive(htms::Template, Debug)]
/// #[template = "fixtures/example.html"]
/// struct Example {}
///
/// // Hook methods are provided on the generated trait `ViewRender`.
/// impl ExampleRender for Example {
///     async fn breaking_news_task() -> String {
///         tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
///         "<p>Some fresh news here :)</p>".to_string()
///     }
///
///     async fn user_dashboard_task() -> String {
///         tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
///         "<p>Awesome user dashboard here :)</p>".to_string()
///     }
/// }
/// ```
///
/// # Example: with context and tasks
///
/// ```rust
/// #[derive(Debug, Clone)]
/// struct State { title: String }
///
/// #[derive(htms::Template, Debug)]
/// #[template = "fixtures/example.html"]
/// struct Example {
///     #[context]    // optional; if the field is named `context`
///     state: State, // must be `Clone`
/// }
///
/// // Hook methods are provided on the generated trait `ViewRender`.
/// impl ExampleRender for Example {
///     async fn breaking_news_task(state: State) -> String {
///         tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
///         format!("<h1>{}</h1><p>Some fresh news here :)</p>", state.title)
///     }
///
///     async fn user_dashboard_task(state: State) -> String {
///         tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
///         format!("<h1>{}</h1><p>Awesome user dashboard here :)</p>", state.title)
///     }
/// }
/// ```
///
/// # Errors
/// This macro emits compile-time errors if:
/// - `#[template = \"...\"]` is missing or not a string literal,
/// - multiple fields are marked `#[context]`,
/// - the chosen context field type does not implement `Clone`.
///
/// # Panics
/// The macro itself does not panic at runtime; it fails at compile-time with diagnostics if misused.
#[proc_macro_derive(Template, attributes(template, context))]
pub fn template_derive(input: TokenStream) -> TokenStream {
    derive::template(&parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(derive::Error::into_compile_error)
}
