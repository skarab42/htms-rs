#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![doc = include_str!("../readme.md")]

pub mod render;
pub mod task;
pub mod template;

pub use bytes::Bytes;
pub use render::Render;
pub use task::Task;
