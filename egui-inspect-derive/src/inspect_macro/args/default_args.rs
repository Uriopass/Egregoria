use super::{expand_to_tokens, InspectFieldArgs};
use darling::FromField;
use quote::quote;

//
// Default arg handling
//
#[derive(Clone, Debug, FromField)]
#[darling(attributes(inspect))]
pub struct InspectFieldArgsDefault {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    name: Option<syn::Ident>,

    #[darling(default)]
    render_trait: Option<syn::Path>,

    #[darling(default)]
    proxy_type: Option<syn::Path>,

    #[darling(default)]
    skip: bool,

    #[darling(default)]
    min_value: Option<f32>,

    #[darling(default)]
    max_value: Option<f32>,

    #[darling(default)]
    step: Option<f32>,

    #[darling(default)]
    header: Option<bool>,

    #[darling(default)]
    indent_children: Option<bool>,
}

impl InspectFieldArgs for InspectFieldArgsDefault {
    fn ident(&self) -> &Option<syn::Ident> {
        &self.ident
    }
    fn ty(&self) -> &syn::Type {
        &self.ty
    }
    fn render_trait(&self) -> &Option<syn::Path> {
        &self.render_trait
    }
    fn name(&self) -> &Option<syn::Ident> {
        &self.name
    }
    fn proxy_type(&self) -> &Option<syn::Path> {
        &self.proxy_type
    }
    fn skip(&self) -> bool {
        self.skip
    }
}

#[derive(Debug)]
pub struct InspectArgs {
    min_value: Option<f32>,
    max_value: Option<f32>,
    step: Option<f32>,
    header: Option<bool>,
    indent_children: Option<bool>,
}

impl From<InspectFieldArgsDefault> for InspectArgs {
    fn from(field_args: InspectFieldArgsDefault) -> Self {
        Self {
            min_value: field_args.min_value,
            max_value: field_args.max_value,
            step: field_args.step,
            header: field_args.header,
            indent_children: field_args.indent_children,
        }
    }
}

impl quote::ToTokens for InspectArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let min_value = expand_to_tokens(&self.min_value);
        let max_value = expand_to_tokens(&self.max_value);
        let step = expand_to_tokens(&self.step);
        let header = expand_to_tokens(&self.header);
        let indent_children = expand_to_tokens(&self.indent_children);

        use quote::TokenStreamExt;
        tokens.append_all(quote!(
            egui_inspect::InspectArgs {
                min_value: #min_value,
                max_value: #max_value,
                step: #step,
                header: #header,
                indent_children: #indent_children,
            }
        ));
    }
}
