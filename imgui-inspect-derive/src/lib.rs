#![allow(missing_copy_implementations, missing_debug_implementations)]
#![deny(
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates
)]

mod inspect_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Inspect, attributes(inspect, inspect_slider, inspect_struct))]
pub fn inspect_macro_derive(input: TokenStream) -> TokenStream {
    inspect_macro::impl_inspect_macro(input)
}
