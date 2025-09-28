use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

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

#[proc_macro_attribute]
pub fn set_up(args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn: syn::ItemFn = match syn::parse(item) {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

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

    let expanded = quote! {
        #input_fn
        ::itest_runner::submit! {
            ::itest_runner::RegisteredSetUp{
                name: #setup_service,
                set_up_fn: ::itest_runner::SetUpFunc::Full(#fn_name),
                deps:  &[#(#dep_strs),*],
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
