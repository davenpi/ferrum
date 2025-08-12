use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Pat, ReturnType, parse_macro_input};

#[proc_macro_attribute]
pub fn task(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;

    // Extract return type from the async function
    let output_type = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => ty.as_ref(),
        ReturnType::Default => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Task functions must have a return type",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract parameters for the new function signature and task struct
    let mut param_names = Vec::new();
    let mut param_types = Vec::new();
    let mut fn_params = Vec::new();

    for input in &input_fn.sig.inputs {
        match input {
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    input,
                    "Task functions cannot have self parameters",
                )
                .to_compile_error()
                .into();
            }
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    let param_name = &pat_ident.ident;
                    let param_type = &pat_type.ty;

                    param_names.push(param_name);
                    param_types.push(param_type);
                    fn_params.push(quote! { #param_name: #param_type });
                } else {
                    return syn::Error::new_spanned(
                        &pat_type.pat,
                        "Only simple parameter names are supported",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    // Generate unique task struct name
    let task_struct_name = syn::Ident::new(
        &format!(
            "{}Task",
            fn_name
                .to_string()
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<String>()
        ),
        fn_name.span(),
    );

    // Generate the transformed function
    let expanded = quote! {
        #fn_vis fn #fn_name(#(#fn_params),*) -> ::ferrum::runtime::TaskHandle<#output_type> {
            // Anonymous task struct that captures parameters
            struct #task_struct_name {
                #(#param_names: #param_types),*
            }

            impl ::ferrum::runtime::Task for #task_struct_name {
                type Output = #output_type;

                fn call(self) -> ::std::pin::Pin<
                    Box<dyn ::std::future::Future<Output = #output_type> + Send>
                > {
                    #(let #param_names = self.#param_names;)*
                    Box::pin(async move #fn_body)
                }
            }

            // Create task with captured parameters and submit
            let task = #task_struct_name { #(#param_names),* };
            ::ferrum::runtime::submit(task)
        }
    };

    TokenStream::from(expanded)
}
