mod inspect_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn inspect_macro_derive(input: TokenStream) -> TokenStream {
    inspect_macro::impl_inspect_macro(input)
}
