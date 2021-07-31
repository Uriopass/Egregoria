#![allow(
    clippy::manual_unwrap_or,
    missing_copy_implementations,
    missing_debug_implementations
)]
#![deny(
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates
)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
