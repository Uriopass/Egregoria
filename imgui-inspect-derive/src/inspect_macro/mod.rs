use darling::FromDeriveInput;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

mod args;
use args::{
    InspectArgsDefault, InspectArgsSlider, InspectFieldArgs, InspectFieldArgsDefault,
    InspectFieldArgsSlider, InspectStructArgs,
};

pub fn impl_inspect_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_args = InspectStructArgs::from_derive_input(&input).unwrap();
    let field_args = parse_field_args(&input);
    generate(&input, struct_args, field_args)
}

struct ParsedField {
    render: proc_macro2::TokenStream,
    render_mut: proc_macro2::TokenStream,
    //skip: bool
}

/// Every trait needs to be checked here
fn handle_inspect_types(parsed_field: &mut Option<ParsedField>, f: &syn::Field) {
    // These are effectively constants
    #[allow(non_snake_case)]
    let INSPECT_DEFAULT_PATH = syn::parse2::<syn::Path>(quote!(inspect)).unwrap();
    #[allow(non_snake_case)]
    let INSPECT_SLIDER_PATH = syn::parse2::<syn::Path>(quote!(inspect_slider)).unwrap();

    // We must check every trait
    try_handle_inspect_type::<InspectFieldArgsSlider, InspectArgsSlider>(
        parsed_field,
        f,
        &INSPECT_SLIDER_PATH,
        quote!(imgui_inspect::InspectRenderSlider),
        quote!(imgui_inspect::InspectArgsSlider),
    );

    try_handle_inspect_type::<InspectFieldArgsDefault, InspectArgsDefault>(
        parsed_field,
        f,
        &INSPECT_DEFAULT_PATH,
        quote!(imgui_inspect::InspectRenderDefault),
        quote!(imgui_inspect::InspectArgsDefault),
    );
}

fn parse_field_args(input: &syn::DeriveInput) -> Vec<ParsedField> {
    match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Parse the fields
                    let parsed_fields: Vec<_> = fields
                        .named
                        .iter()
                        .map(|f| {
                            let mut parsed_field: Option<ParsedField> = None;

                            handle_inspect_types(&mut parsed_field, f);

                            if parsed_field.is_none() {
                                handle_inspect_type::<InspectFieldArgsDefault, InspectArgsDefault>(
                                    &mut parsed_field,
                                    f,
                                    quote!(imgui_inspect::InspectRenderDefault),
                                    quote!(imgui_inspect::InspectArgsDefault),
                                );
                            }

                            parsed_field.unwrap()
                        })
                        .collect();

                    parsed_fields
                }
                Fields::Unnamed(_) => unimplemented!(
                    "#[derive(Inspect)] is only allowed on structs with named fields."
                ),
                Fields::Unit => vec![],
            }
        }
        _ => unimplemented!(),
    }
}

fn try_handle_inspect_type<
    FieldArgsT: darling::FromField + InspectFieldArgs + Clone,
    ArgsT: From<FieldArgsT> + ToTokens,
>(
    parsed_field: &mut Option<ParsedField>,
    f: &syn::Field,
    path: &syn::Path,
    default_render_trait: proc_macro2::TokenStream,
    arg_type: proc_macro2::TokenStream,
) {
    if f.attrs.iter().any(|x| x.path == *path) {
        handle_inspect_type::<FieldArgsT, ArgsT>(parsed_field, f, default_render_trait, arg_type);
    }
}

// Does common data gathering and error checking, then calls create_render_call and create_render_mut_call to emit
// code for inspecting.
fn handle_inspect_type<
    FieldArgsT: darling::FromField + InspectFieldArgs + Clone,
    ArgsT: From<FieldArgsT> + ToTokens,
>(
    parsed_field: &mut Option<ParsedField>,
    f: &syn::Field,
    default_render_trait: proc_macro2::TokenStream,
    arg_type: proc_macro2::TokenStream,
) {
    //TODO: Improve error message
    if parsed_field.is_some() {
        panic!(
            "Too many inspect attributes on a single member {:?}",
            f.ident
        );
    }

    let field_args = FieldArgsT::from_field(f).unwrap();

    if field_args.skip() {
        *parsed_field = Some(ParsedField {
            render: quote!(),
            render_mut: quote!(),
            //skip: true
        });

        return;
    }

    let render_trait = match field_args.render_trait() {
        Some(t) => t.clone(),
        None => syn::parse2::<syn::Path>(default_render_trait).unwrap(),
    };

    let arg_type = syn::parse2::<syn::Type>(arg_type).unwrap();
    let args: ArgsT = field_args.clone().into();

    let render = create_render_call(
        field_args.ident().as_ref().unwrap(),
        field_args.name(),
        field_args.ty(),
        &render_trait,
        field_args.proxy_type(),
        &arg_type,
        &args,
    );

    let render_mut = create_render_mut_call(
        field_args.ident().as_ref().unwrap(),
        field_args.name(),
        field_args.ty(),
        field_args.on_set(),
        &render_trait,
        field_args.proxy_type(),
        &arg_type,
        &args,
    );

    *parsed_field = Some(ParsedField {
        render,
        render_mut,
        //skip: false
    });
}

fn create_render_call<T: ToTokens>(
    field_name: &syn::Ident,
    field_rename: &Option<syn::Ident>,
    field_type: &syn::Type,
    render_trait: &syn::Path,
    proxy_type: &Option<syn::Path>,
    arg_type: &syn::Type,
    args: &T,
) -> proc_macro2::TokenStream {
    use quote::format_ident;
    let args_name1 = format_ident!("_inspect_args_{}", field_name);
    let args_name2 = args_name1.clone();

    let field_name1 = field_name.clone();
    let field_name2 = field_rename.clone().unwrap_or_else(|| field_name.clone());

    let source_type = if let Some(w) = proxy_type {
        quote!(#w)
    } else {
        quote!(#field_type)
    };

    quote! {{
        #[allow(non_upper_case_globals)]
        const #args_name1 : #arg_type = #args;
        let values : Vec<_> = data.iter().map(|x| &x.#field_name1).collect();
        if data.len() != 0 {
            <#source_type as #render_trait<#field_type>>::render(values.as_slice(), stringify!(#field_name2), ui, &#args_name2);
        }
    }}
}

fn create_render_mut_call<T: ToTokens>(
    field_name: &syn::Ident,
    field_rename: &Option<syn::Ident>,
    field_type: &syn::Type,
    on_set: &Option<syn::Ident>,
    render_trait: &syn::Path,
    proxy_type: &Option<syn::Path>,
    arg_type: &syn::Type,
    args: &T,
) -> proc_macro2::TokenStream {
    use quote::format_ident;
    let args_name1 = format_ident!("_inspect_args_{}", field_name);
    let args_name2 = args_name1.clone();

    let field_name1 = field_name.clone();
    let field_name2 = field_rename.clone().unwrap_or_else(|| field_name.clone());

    let source_type = if let Some(w) = proxy_type {
        quote!(#w)
    } else {
        quote!(#field_type)
    };

    let on_set_callback_impl = match on_set {
        Some(ident) => quote! {{
            for d in data.iter_mut() {
                d.#ident();
            }
        }},
        None => quote! {{}},
    };

    quote! {{
        #[allow(non_upper_case_globals)]
        const #args_name1 : #arg_type = #args;
        let mut values : Vec<_> = data.iter_mut().map(|x| &mut x.#field_name1).collect();
        let mut changed = <#source_type as #render_trait<#field_type>>::render_mut(&mut values.as_mut_slice(), stringify!(#field_name2), ui, &#args_name2);

        #on_set_callback_impl

        _has_any_field_changed |= changed;
    }}
}

// Provide a way to early out and generate no code. It's going to be a common case for
// downstream users to want to only conditionally generate code, and it's easier to do this
// by adding an early-out here that can be configured via a cargo feature, than having to
// mark up all the downstream code with conditional compile directives.
#[cfg(not(feature = "generate_code"))]
fn generate(
    input: &syn::DeriveInput,
    struct_args: InspectStructArgs,
    parsed_fields: Vec<ParsedField>,
) -> proc_macro::TokenStream {
    return proc_macro::TokenStream::from(quote! {});
}

#[cfg(feature = "generate_code")]
fn generate(
    input: &syn::DeriveInput,
    struct_args: InspectStructArgs,
    parsed_fields: Vec<ParsedField>,
) -> proc_macro::TokenStream {
    let struct_name1 = &struct_args.ident;
    let struct_name2 = &struct_args.ident;
    let struct_name3 = &struct_args.ident;
    let struct_name4 = &struct_args.ident;
    let struct_name5 = &struct_args.ident;
    let struct_name6 = &struct_args.ident;

    let mut render_impls = vec![];
    let mut render_mut_impls = vec![];

    for parsed_field in parsed_fields {
        render_impls.push(parsed_field.render);
        render_mut_impls.push(parsed_field.render_mut);
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let default_impl = quote! {

        impl #impl_generics imgui_inspect::InspectRenderDefault<#struct_name1> for #struct_name2 #ty_generics #where_clause {
            fn render(data: &[&Self], label: &'static str, ui: &imgui_inspect::imgui::Ui, args: &imgui_inspect::InspectArgsDefault) {
                <#struct_name3 as imgui_inspect::InspectRenderStruct<#struct_name4>>::render(data, label, ui, &imgui_inspect::InspectArgsStruct { header: args.header, indent_children: args.indent_children })
            }

            fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui_inspect::imgui::Ui, args: &imgui_inspect::InspectArgsDefault) -> bool {
                <#struct_name5 as imgui_inspect::InspectRenderStruct<#struct_name6>>::render_mut(data, label, ui, &imgui_inspect::InspectArgsStruct { header: args.header, indent_children: args.indent_children })
            }
        }
    };

    let struct_impl = quote! {
        impl #impl_generics imgui_inspect::InspectRenderStruct<#struct_name1> for #struct_name2 #ty_generics #where_clause {
            fn render(data: &[&Self], label: &'static str, ui: &imgui_inspect::imgui::Ui, args: &imgui_inspect::InspectArgsStruct) {
                let header_name = stringify!(#struct_name3);

                let mut header = true;
                if let Some(h) = args.header {
                    header = h;
                }

                let mut indent_children = true;
                if let Some(ic) = args.indent_children {
                    header = ic;
                }

                let should_render_children = if header {
                    imgui_inspect::imgui::CollapsingHeader::new(&imgui_inspect::imgui::im_str!( "{}", label)).default_open(true).build(&ui)
                } else {
                    true
                };

                if should_render_children {
                    let id_token = ui.push_id(label);
                    if indent_children { ui.indent(); }
                    #(
                        #render_impls
                    )*
                    if indent_children { ui.unindent(); }
                    id_token.pop(ui);
                }
            }

            fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui_inspect::imgui::Ui, args: &imgui_inspect::InspectArgsStruct) -> bool {
                let header_name = stringify!(#struct_name4);

                let mut header = true;
                if let Some(h) = args.header {
                    header = h;
                }

                let mut indent_children = true;
                if let Some(ic) = args.indent_children {
                    indent_children = ic;
                }

                let should_render_children = if header {
                    imgui_inspect::imgui::CollapsingHeader::new(&imgui_inspect::imgui::im_str!("{}", label)).default_open(true).build(&ui)
                } else {
                    true
                };

                let mut _has_any_field_changed = false;
                if should_render_children {
                    let id_token = ui.push_id(label);
                    if indent_children { ui.indent(); }
                    #(
                        #render_mut_impls
                    )*
                    if indent_children { ui.unindent(); }
                    id_token.pop(ui);
                }
                _has_any_field_changed
            }
        }
    };

    proc_macro::TokenStream::from(quote! {
        #default_impl
        #struct_impl
    })
}
