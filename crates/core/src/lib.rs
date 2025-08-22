#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![doc = include_str!("../readme.md")]

#[cfg(feature = "axum")]
pub mod axum;

pub mod render;
pub mod task;
pub mod template;

pub use bytes::Bytes;
pub use render::Render;
pub use task::Task;
