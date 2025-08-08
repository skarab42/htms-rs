#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![doc = include_str!("../readme.md")]

pub mod prelude {
    pub use crate::core;
}

#[must_use]
pub const fn core() -> u8 {
    42
}

#[cfg(test)]
mod tests {
    use crate::core;

    #[test]
    fn it_works() {
        assert_eq!(core(), 42);
    }
}
