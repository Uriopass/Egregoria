use super::*;
use darling::FromField;
use quote::quote;

//
// Struct arg handling
//
#[derive(Clone, Debug, FromField)]
#[darling(attributes(inspect_struct))]
pub struct InspectFieldArgsStruct {
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

    //TODO: Default to true
    #[darling(default)]
    header: Option<bool>,

    //TODO: Default to true
    #[darling(default)]
    indent_children: Option<bool>,
}

impl InspectFieldArgs for InspectFieldArgsStruct {
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
pub struct InspectArgsStruct {
    //TODO: Default to true
    pub header: Option<bool>,

    //TODO: Default to true
    pub indent_children: Option<bool>,
}

impl From<InspectFieldArgsStruct> for InspectArgsStruct {
    fn from(field_args: InspectFieldArgsStruct) -> Self {
        Self {
            header: field_args.header,
            indent_children: field_args.indent_children,
        }
    }
}

impl quote::ToTokens for InspectArgsStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let header = expand_to_tokens(&self.header);
        let indent_children = expand_to_tokens(&self.indent_children);

        use quote::TokenStreamExt;
        tokens.append_all(quote!(
            imgui_inspect::InspectArgsStruct {
                header: #header,
                indent_children: #indent_children,
            }
        ));
    }
}
