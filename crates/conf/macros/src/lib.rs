//! This crate provides a derive macro that provides the necessary
//! functionality for a struct to be used as a configuration object and be
//! bundled into.

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Pat, Token, Type};

struct EnsureArgs(syn::Expr, syn::LitStr);

impl Parse for EnsureArgs {
    fn parse(input: ParseStream) -> Result<EnsureArgs, syn::Error> {
        let condition = input.parse()?;
        input.parse::<Token![,]>()?;
        let string = input.parse()?;
        Ok(EnsureArgs(condition, string))
    }
}

#[derive(Clone)]
struct ConfigField {
    check_fns: Vec<Option<proc_macro2::TokenStream>>,
    field_name: syn::Ident,
    field_type: syn::Type,
    field_publicity: syn::Visibility,
}

#[proc_macro_derive(Config, attributes(check, ensure, is_finite))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let data = &input.data;

    let mut config_fields: Vec<ConfigField> = Vec::new();

    let struct_publicity = &input.vis;
    let mut is_finite_check = false;

    if let Data::Struct(data_struct) = data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let mut field_info = ConfigField {
                    check_fns: Vec::new(),
                    field_name: field.ident.clone().unwrap(),
                    field_type: field.ty.clone(),
                    field_publicity: field.vis.clone(),
                };

                let field_name = &field_info.field_name;
                let field_type = &field_info.field_type;

                let inner_check_fns = &mut field_info.check_fns;
                for attr in &field.attrs {
                    // get the check attribute with closures like this: #[check(|v| v > 0)]
                    if attr.path.is_ident("check") {
                        if let Err(error) = check_attr(field_type, inner_check_fns, attr) {
                            return error;
                        };
                    } else if attr.path.is_ident("ensure") {
                        if let Err(error) =
                            ensure_attr(field_name, field_type, inner_check_fns, attr)
                        {
                            return error;
                        };
                    } else if attr.path.is_ident("is_finite") {
                        is_finite_check = !is_finite_check;
                    }

                    if is_finite_check {
                        let check_fn = Some(quote! {
                            (|#field_name: &#field_type| {
                                Ok(ensure!(#field_name.is_finite(), "`{}` is not finite. It must be a finite Number with type: {}", stringify!(#field_name), stringify!(#field_type)))
                            })
                        });

                        inner_check_fns.push(check_fn)
                    }
                }

                config_fields.push(field_info);
            }
        } else {
            return syn::Error::new_spanned(
                &data_struct.fields,
                "expected a struct with named fields",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return match data {
            Data::Enum(data_enum) => {
                let enum_tokens = data_enum.enum_token;
                syn::Error::new_spanned(enum_tokens, "expected a struct, not an enum")
                    .to_compile_error()
                    .into()
            }
            Data::Union(data_union) => {
                let union_token = data_union.union_token;
                syn::Error::new_spanned(union_token, "expected a struct, not a union")
                    .to_compile_error()
                    .into()
            }
            _ => syn::Error::new_spanned(struct_name, "expected a struct")
                .to_compile_error()
                .into(),
        };
    }

    /* We need to generate the following code:
     * - a function to check all fields
     * - a function to deserialize from config and fill default values while
     *   also checking
     * - a partial struct with all the fields that are options
     * - derive the Deserialize & Serialize trait for the partial struct
     * - a TryFrom implementation for the partial struct
     */

    let mut check_all_fn_impl = Vec::new();

    for (field_name, check_fns) in config_fields.iter().map(|f| (&f.field_name, &f.check_fns)) {
        for check_fn in check_fns {
            let check_fn = check_fn.as_ref();

            if let Some(check_fn) = check_fn {
                check_all_fn_impl.push(quote! {
                match #check_fn(&self.#field_name) {
                    Ok(_) => (),
                    Err(e) => errors.push(
                            (format!("{} failed check. value {:?} did not pass closure {}, Error: {}", stringify!(#field_name), self.#field_name, stringify!(#check_fn), e),
                                e.to_string())
                        ),
                }
            });
            }
        }
    }

    let partial_struct = Ident::new(&format!("Partial{}", struct_name), Span::call_site());
    let field_names = config_fields
        .iter()
        .map(|f| f.field_name.clone())
        .collect_vec();
    let field_publicity = config_fields
        .iter()
        .map(|f| f.field_publicity.clone())
        .collect_vec();
    let field_types = config_fields
        .iter()
        .map(|f| f.field_type.clone())
        .collect_vec();

    let expanded = quote! {
        impl #struct_name {
            fn check(&self) -> Result<(), Vec<(std::string::String, std::string::String)>> {
                let mut errors: Vec<(std::string::String, std::string::String)> = Vec::new();

                // define all values of struct fields
                #(let #field_names = &self.#field_names;)*

                #(#check_all_fn_impl)*

                if errors.len() > 0 {
                    Err(errors)
                } else {
                    Ok(())
                }
            }
        }

        // Create a partial version of the struct with options for every field
        // for deserializing.
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        #struct_publicity struct #partial_struct {
            #(#field_publicity #field_names: Option<#field_types>,)*
        }

        impl Into<#struct_name> for #partial_struct {
            fn into(self) -> #struct_name {
                let defaults = #struct_name::default();

                #struct_name {
                    #(#field_names: self.#field_names.unwrap_or(defaults.#field_names),)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn ensure_attr(
    field_name: &Ident,
    field_type: &Type,
    inner_check_fns: &mut Vec<Option<proc_macro2::TokenStream>>,
    attr: &Attribute,
) -> Result<(), TokenStream> {
    // the ensure attribute has 2 inputs like this:
    // #[ensure(move_margin > 0., "`move_margin` must be positive.")]
    // it uses the `ensure!()` macro from the anyhow crate

    // We need to get the two inputs separated by a comma separately. The first
    // one is a boolean expression and the second one is a string literal.

    // convert to tuple
    let ensure_args = match attr.parse_args::<EnsureArgs>() {
        Ok(args) => args,
        Err(e) => {
            let mut error = syn::Error::new_spanned(
                attr.tokens.to_token_stream(),
                "expected inputs like: `#[ensure(move_margin > 0., \"`move_margin` must be positive.\")]`",
            );
            error.combine(e);
            return Err(error.to_compile_error().into());
        }
    };

    let arg_type = quote! { &#field_type };

    let condition = ensure_args.0;
    let string = ensure_args.1;

    let check_fn = Some(quote! {
        (|#field_name: #arg_type| {
            ensure!(#condition, #string);
            Ok(())
        })
    });

    inner_check_fns.push(check_fn);
    Ok(())
}

fn check_attr(
    field_type: &Type,
    inner_check_fns: &mut Vec<Option<proc_macro2::TokenStream>>,
    attr: &Attribute,
) -> Result<(), TokenStream> {
    let closure = match attr.parse_args::<syn::ExprClosure>() {
        Ok(args) => args,
        Err(e) => {
            let mut error =
                syn::Error::new_spanned(attr, "expected a closure like: `#[check(|v| v > 0)]`");
            error.combine(e);
            return Err(error.to_compile_error().into());
        }
    };

    // The closure should take a reference to the field type and return a
    // Result<(), String>
    if closure.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            closure.inputs,
            "expected a closure with one argument",
        )
        .to_compile_error()
        .into());
    }

    let arg = &closure.inputs[0];
    let Pat::Type(arg) = arg else {
        return Err(syn::Error::new_spanned(
            attr,
            "expected a closure with one argument",
        )
            .to_compile_error()
            .into());
    };

    let arg_type = quote! { &#field_type };

    let arg_name = &arg.pat;
    let arg_name = quote! { #arg_name };

    let body = &closure.body;
    let check_fn = Some(quote! {
        (|#arg_name: #arg_type| {
            #body
        })
    });

    inner_check_fns.push(check_fn);
    Ok(())
}
