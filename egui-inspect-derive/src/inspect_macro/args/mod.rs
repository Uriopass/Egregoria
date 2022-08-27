mod default_args;
mod slider_args;
mod struct_args;

use darling::FromDeriveInput;
use quote::quote;

pub use default_args::InspectArgsDefault;
pub use default_args::InspectFieldArgsDefault;
pub use slider_args::InspectArgsSlider;
pub use slider_args::InspectFieldArgsSlider;
pub use struct_args::InspectArgsStruct;
pub use struct_args::InspectFieldArgsStruct;

// Utility function to convert an Option<T> to tokens
pub fn expand_to_tokens<T: quote::ToTokens>(input: &Option<T>) -> proc_macro2::TokenStream {
    match input {
        Some(value) => quote!(Some(#value)),
        None => quote!(None),
    }
}

// Metadata from the struct's type annotation
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inspect))]
pub struct InspectStructArgs {
    pub ident: syn::Ident,
}

// We support multiple distinct inspect annotations (i.e. inspect_slider, inspect_text)
// Each distinct type will have a struct for capturing the metadata. These metadata structs
// must implement this trait
pub trait InspectFieldArgs {
    fn ident(&self) -> &Option<syn::Ident>;
    fn ty(&self) -> &syn::Type;
    fn render_trait(&self) -> &Option<syn::Path>;
    fn name(&self) -> &Option<syn::Ident>;
    fn proxy_type(&self) -> &Option<syn::Path>;
    fn skip(&self) -> bool;
}
