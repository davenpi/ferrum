use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn task(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;
    let fn_vis = &input_fn.vis;
    
    // Extract return type from async fn
    let output_type = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => quote! { #ty },
        ReturnType::Default => quote! { () },
    };
    
    // Generate the new function that returns a TaskWrapper
    let expanded = quote! {
        #fn_vis fn #fn_name(#fn_inputs) -> impl ::ferrum::runtime::Task<Output = #output_type> {
            ::ferrum::runtime::TaskWrapper::new(|| async move #fn_body)
        }
    };
    
    TokenStream::from(expanded)
}
