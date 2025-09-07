use proc_macro::TokenStream;
use quote::quote;




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
