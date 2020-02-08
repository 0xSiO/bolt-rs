extern crate proc_macro;

use proc_macro::TokenStream;

use syn::{Lit, NestedMeta};

use quote::quote;

#[proc_macro_attribute]
pub fn bolt_version(attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(attr_args as syn::AttributeArgs);
    let signature = &mut func.sig;
    let function_body = &func.block;
    let attributes = &func.attrs;
    let visibility = func.vis;

    let versions: Vec<u32> = args
        .into_iter()
        .map(|item| {
            if let NestedMeta::Lit(lit) = item {
                if let Lit::Int(lit_int) = lit {
                    lit_int
                        .base10_parse::<u32>()
                        .expect("couldn't parse version")
                } else {
                    panic!("Invalid version token: {:?}", lit);
                }
            } else {
                panic!("Invalid version token: {:?}", item);
            }
        })
        .collect();

    let gen = quote! {
        #(#attributes)*
        #visibility #signature {
            if [#(#versions),*].contains(self.version) {
                #function_body
            } else {
                Err(crate::error::ClientError::UnsupportedOperation(self.version).into())
            }
        }
    };
    gen.into()
}
