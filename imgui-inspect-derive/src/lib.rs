#![deny(
    rustdoc::all,
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates,
    clippy::all,
    clippy::doc_markdown,
    clippy::wildcard_imports
)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::blocks_in_if_conditions,
    clippy::upper_case_acronyms,
    clippy::must_use_candidate,
    missing_copy_implementations,
    missing_debug_implementations
)]

mod inspect_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Inspect, attributes(inspect, inspect_slider, inspect_struct))]
pub fn inspect_macro_derive(input: TokenStream) -> TokenStream {
    inspect_macro::impl_inspect_macro(input)
}
