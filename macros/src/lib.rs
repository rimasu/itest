use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Ident, ReturnType, spanned::Spanned};

#[proc_macro_attribute]
pub fn itest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn: syn::ItemFn = syn::parse(item).unwrap();
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let expanded = quote! {
        #input_fn

        ::itest_runner::submit! {
            ::itest_runner::RegisteredITest{
                name: #fn_name_str,
                test_fn: #fn_name
            }
        }
    };

    expanded.into()
}

fn is_unit_result(return_type: &ReturnType) -> Result<bool, Error> {
    match return_type {
        syn::ReturnType::Default => Err(Error::new(return_type.span(), "expect a return type")),
        syn::ReturnType::Type(_, t) => match &**t {
            syn::Type::Path(type_path) => {
                // Get the last segment (should be "Result")
                let last_segment = type_path.path.segments.last();

                match last_segment {
                    Some(segment) if segment.ident == "Result" => {
                        // Extract the generic arguments
                        match &segment.arguments {
                            syn::PathArguments::AngleBracketed(args) => {
                                // First generic argument is the Ok type
                                if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first()
                                {
                                    Ok(match ok_type {
                                        syn::Type::Tuple(tuple) if tuple.elems.is_empty() => true, // is unit,
                                        _ => false, // It's a concrete type
                                    })
                                } else {
                                    Err(Error::new(
                                        segment.span(),
                                        "Result must have generic arguments",
                                    ))
                                }
                            }
                            _ => Err(Error::new(
                                segment.span(),
                                "Result must have angle-bracketed arguments",
                            )),
                        }
                    }
                    _ => Err(Error::new(type_path.span(), "Expected Result return type")),
                }
            }
            _ => Err(Error::new(t.span(), "Expected a Result output")),
        },
    }
}

#[proc_macro_attribute]
pub fn set_up(args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn: syn::ItemFn = match syn::parse(item) {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    let is_async = input_fn.sig.asyncness.is_some();
    let is_unit_result = match is_unit_result(&input_fn.sig.output) {
        Ok(flag) => flag,
        Err(e) => return e.to_compile_error().into(),
    };

    let span = input_fn.span().unwrap();
    let file = span.file();
    let line = span.line();

    let name: syn::Ident = match syn::parse(args) {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    let mut dependencies = Vec::new();
    for attr in &input_fn.attrs {
        if attr.path().is_ident("depends_on") {
            let depends_on = attr.parse_args::<Ident>().unwrap().to_string();
            dependencies.push(depends_on);
        }
    }

    let setup_service = name.to_string();

    let dep_strs: Vec<proc_macro2::TokenStream> = dependencies
        .into_iter()
        .map(|dep_str| {
            quote! { #dep_str }
        })
        .collect();

    let fn_name = &input_fn.sig.ident;
    let wrapper_name = Ident::new(&format!("__{}_set_up_wrapper", fn_name), fn_name.span());

    let wrapper_fn = if is_async {
        if is_unit_result {
            quote! {
                fn #wrapper_name(ctx: ::itest_runner::Context) -> ::itest_runner::SetFnOutput {
                    Box::pin(async move {
                        match #fn_name(ctx).await {
                            Ok(teardown) => Ok(None),
                            Err(e) => Err(e),
                        }
                    })
                }
            }
        } else {
            quote! {
                fn #wrapper_name(ctx: ::itest_runner::Context) -> ::itest_runner::SetFnOutput {
                    Box::pin(async move {
                        match #fn_name(ctx).await {
                            Ok(teardown) => Ok(Some(Box::new(teardown) as Box<dyn TearDown>)),
                            Err(e) => Err(e),
                        }
                    })
                }
            }
        }
     } else {
        quote! {
            fn #wrapper_name(ctx: ::itest_runner::Context) -> ::itest_runner::SetFnOutput {
                Box::pin(async move {
                    match #fn_name(ctx) {
                        Ok(teardown) => Ok(None),
                        Err(e) => Err(e),
                    }
                })
            }
        }
    };

    let expanded = quote! {

        #input_fn

        #wrapper_fn

        ::itest_runner::submit! {
             ::itest_runner::RegisteredSetUp{
                name: #setup_service,
                set_up_fn: #wrapper_name,
                deps:  &[#(#dep_strs),*],
                file: #file,
                line: #line,
            }
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn depends_on(_args: TokenStream, input: TokenStream) -> TokenStream {
    // The depends attribute is handled by the set_up macro
    // This is just a placeholder to make the attribute valid
    input
}
