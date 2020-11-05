use super::*;
use darling::FromField;
use quote::quote;

//
// Slider arg handling
//
#[derive(Clone, Debug, FromField)]
#[darling(attributes(inspect_slider))]
pub struct InspectFieldArgsSlider {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    render_trait: Option<syn::Path>,

    #[darling(default)]
    proxy_type: Option<syn::Path>,

    #[darling(default)]
    on_set: Option<syn::Ident>,

    #[darling(default)]
    skip: bool,

    #[darling(default)]
    min_value: Option<f32>,

    #[darling(default)]
    max_value: Option<f32>,
}

impl InspectFieldArgs for InspectFieldArgsSlider {
    fn ident(&self) -> &Option<syn::Ident> {
        &self.ident
    }
    fn ty(&self) -> &syn::Type {
        &self.ty
    }
    fn render_trait(&self) -> &Option<syn::Path> {
        &self.render_trait
    }
    fn proxy_type(&self) -> &Option<syn::Path> {
        &self.proxy_type
    }
    fn on_set(&self) -> &Option<syn::Ident> {
        &self.on_set
    }
    fn skip(&self) -> bool {
        self.skip
    }
}

#[derive(Debug)]
pub struct InspectArgsSlider {
    min_value: Option<f32>,
    max_value: Option<f32>,
}

impl From<InspectFieldArgsSlider> for InspectArgsSlider {
    fn from(field_args: InspectFieldArgsSlider) -> Self {
        Self {
            min_value: field_args.min_value,
            max_value: field_args.max_value,
        }
    }
}

impl quote::ToTokens for InspectArgsSlider {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let min_value = expand_to_tokens(&self.min_value);
        let max_value = expand_to_tokens(&self.max_value);

        use quote::TokenStreamExt;
        tokens.append_all(quote!(
            imgui_inspect::InspectArgsSlider {
                min_value: #min_value,
                max_value: #max_value,
            }
        ));
    }
}
