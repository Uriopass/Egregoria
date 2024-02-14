use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsUnnamed};

mod args;
use args::{InspectArgs, InspectFieldArgs, InspectFieldArgsDefault, InspectStructArgs};

pub fn impl_inspect_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_args = InspectStructArgs::from_derive_input(&input).unwrap();
    let field_args = parse_field_args(&input, &struct_args);
    generate(&input, struct_args, field_args)
}

struct ParsedField {
    render: TokenStream,
    render_mut: TokenStream,
    //skip: bool
}

/// Every trait needs to be checked here
fn handle_inspect_types(parsed_field: &mut Option<ParsedField>, f: &syn::Field) {
    // These are effectively constants
    #[allow(non_snake_case)]
    let INSPECT_DEFAULT_PATH = syn::parse2::<syn::Path>(quote!(inspect)).unwrap();

    try_handle_inspect_type::<InspectFieldArgsDefault, InspectArgs>(
        parsed_field,
        f,
        &INSPECT_DEFAULT_PATH,
        quote!(egui_inspect::Inspect),
        quote!(egui_inspect::InspectArgs),
    );
}

fn parse_field_args(input: &DeriveInput, struct_args: &InspectStructArgs) -> Vec<ParsedField> {
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
                                handle_inspect_type::<InspectFieldArgsDefault, InspectArgs>(
                                    &mut parsed_field,
                                    f,
                                    quote!(egui_inspect::Inspect),
                                    quote!(egui_inspect::InspectArgs),
                                );
                            }

                            parsed_field.unwrap()
                        })
                        .collect();

                    parsed_fields
                }
                Fields::Unnamed(ref field) => {
                    if field.unnamed.len() != 1 {
                        panic!("Unnamed fields with 2 or more fields are not supported");
                    }
                    vec![ParsedField {
                        render: create_render_call_unit_struct(field),
                        render_mut: create_render_mut_call_unit_struct(field),
                    }]
                }
                Fields::Unit => vec![],
            }
        }
        Data::Enum(ref data) => {
            vec![ParsedField {
                render: create_render_call_enum(data, struct_args),
                render_mut: create_render_mut_call_enum(data, struct_args),
            }]
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
    default_render_trait: TokenStream,
    arg_type: TokenStream,
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
    default_render_trait: TokenStream,
    arg_type: TokenStream,
) {
    if parsed_field.is_some() {
        panic!(
            "Too many debug_inspect attributes on a single member {:?}",
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

fn create_render_call_unit_struct(data: &FieldsUnnamed) -> TokenStream {
    let ty = data.unnamed.iter().next().unwrap();
    let ty = &ty.ty;

    quote! {{
        <#ty as egui_inspect::Inspect<#ty>>::render(&data.0, "", ui, args)
    }}
}

fn create_render_mut_call_unit_struct(data: &FieldsUnnamed) -> TokenStream {
    let ty = data.unnamed.iter().next().unwrap();
    let ty = &ty.ty;

    quote! {{
        <#ty as egui_inspect::Inspect<#ty>>::render_mut(&mut data.0, "", ui, args)
    }}
}

fn create_render_call_enum(data: &syn::DataEnum, args: &InspectStructArgs) -> TokenStream {
    let variants = data.variants.iter().map(|v| {
        let variant_name = &v.ident;
        if !v.fields.is_empty() {
            panic!("only simple enums are supported")
        }
        variant_name
    });

    let sname = &args.ident;

    quote! {{
        match data {
            #(#sname::#variants => {
                ui.label(stringify!(#variants));
            })*
        }
    }}
}

fn create_render_mut_call_enum(data: &syn::DataEnum, args: &InspectStructArgs) -> TokenStream {
    let variants = data.variants.iter().map(|v| {
        let variant_name = &v.ident;
        if !v.fields.is_empty() {
            panic!("only simple enums are supported")
        }
        variant_name
    });

    let sname = &args.ident;

    quote! {{
        match data {
            #(#sname::#variants => {
                ui.label(stringify!(#variants));
            })*
        }
    }}
}

fn create_render_call<T: ToTokens>(
    field_name: &syn::Ident,
    field_rename: &Option<syn::Ident>,
    field_type: &syn::Type,
    render_trait: &syn::Path,
    proxy_type: &Option<syn::Path>,
    arg_type: &syn::Type,
    args: &T,
) -> TokenStream {
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
        let value = &data.#field_name1;
        <#source_type as #render_trait<#field_type>>::render(value, stringify!(#field_name2), ui, &#args_name2);
    }}
}

fn create_render_mut_call<T: ToTokens>(
    field_name: &syn::Ident,
    field_rename: &Option<syn::Ident>,
    field_type: &syn::Type,
    render_trait: &syn::Path,
    proxy_type: &Option<syn::Path>,
    arg_type: &syn::Type,
    args: &T,
) -> TokenStream {
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
        let mut value = &mut data.#field_name1;
        let mut changed = <#source_type as #render_trait<#field_type>>::render_mut(value, stringify!(#field_name2), ui, &#args_name2);

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
    input: &DeriveInput,
    struct_args: InspectStructArgs,
    parsed_fields: Vec<ParsedField>,
) -> proc_macro::TokenStream {
    let struct_name1 = &struct_args.ident;
    let struct_name2 = &struct_args.ident;
    let struct_name3 = &struct_args.ident;
    let struct_name4 = &struct_args.ident;

    let mut render_impls = vec![];
    let mut render_mut_impls = vec![];

    for parsed_field in parsed_fields {
        render_impls.push(parsed_field.render);
        render_mut_impls.push(parsed_field.render_mut);
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let struct_impl = quote! {
        impl #impl_generics #struct_name2 #ty_generics #where_clause {
            fn impls(data: &Self, ui: &mut egui_inspect::egui::Ui, args: &egui_inspect::InspectArgs, header: bool, indent_children: bool) {
                #(#render_impls)*
            }

            fn impls_mut(data: &mut Self, ui: &mut egui_inspect::egui::Ui, args: &egui_inspect::InspectArgs, header: bool, indent_children: bool) -> bool {
                let mut _has_any_field_changed = false;
                #(#render_mut_impls)*
                ;_has_any_field_changed
            }
        }

        impl #impl_generics egui_inspect::Inspect<#struct_name1> for #struct_name2 #ty_generics #where_clause {
            fn render(data: &Self, label: &'static str, ui: &mut egui_inspect::egui::Ui, args: &egui_inspect::InspectArgs) {
                let header_name = stringify!(#struct_name3);

                let mut header = true;
                if let Some(h) = args.header {
                    header = h;
                }

                let mut indent_children = true;
                if let Some(ic) = args.indent_children {
                    header = ic;
                }

                if header {
                    egui_inspect::egui::CollapsingHeader::new(label).default_open(true).show(ui, |ui| {
                        Self::impls(data, ui, args, header, indent_children);
                    });
                } else {
                    Self::impls(data, ui, args, header, indent_children);
                };
            }

            fn render_mut(data: &mut Self, label: &'static str, ui: &mut egui_inspect::egui::Ui, args: &egui_inspect::InspectArgs) -> bool {
                let header_name = stringify!(#struct_name4);

                let mut header = true;
                if let Some(h) = args.header {
                    header = h;
                }

                let mut indent_children = true;
                if let Some(ic) = args.indent_children {
                    indent_children = ic;
                }


                let mut _has_any_field_changed = false;
                if header {
                    egui_inspect::egui::CollapsingHeader::new(label).default_open(true).show(ui, |ui| {
                        _has_any_field_changed = Self::impls_mut(data, ui, args, header, indent_children);
                    });
                } else {
                    _has_any_field_changed = Self::impls_mut(data, ui, args, header, indent_children);
                };

                _has_any_field_changed
            }
        }
    };

    proc_macro::TokenStream::from(quote! {
        #struct_impl
    })
}
