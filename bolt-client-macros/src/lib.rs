#![warn(rust_2018_idioms)]

use proc_macro::TokenStream;

use syn::*;

use quote::quote;

fn get_fn_info(
    func: &ItemFn,
    args: AttributeArgs,
) -> (&Vec<Attribute>, &Visibility, &Signature, Vec<u32>, &Block) {
    let signature = &func.sig;
    let function_body = &func.block;
    let attributes = &func.attrs;
    let visibility = &func.vis;
    let versions: Vec<u32> = args
        .into_iter()
        .map(|item| {
            if let NestedMeta::Lit(lit) = item {
                match lit {
                    Lit::Int(lit_int) => lit_int
                        .base10_parse::<u32>()
                        .expect("couldn't parse version"),
                    Lit::Float(lit_float) => {
                        let version = lit_float
                            .base10_parse::<f64>()
                            .expect("couldn't parse version");
                        let major = version.trunc() as u32;
                        let minor = (version.fract() * 10.0).round() as u32;
                        minor << 8 | major
                    }
                    _ => panic!("invalid version token: {:?}", lit),
                }
            } else {
                panic!("invalid version token: {:?}", item);
            }
        })
        .collect();
    (attributes, visibility, signature, versions, function_body)
}

#[proc_macro_attribute]
pub fn bolt_version(attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(attr_args as syn::AttributeArgs);
    let (attributes, visibility, signature, versions, function_body) = get_fn_info(&func, args);

    quote!(
        #(#attributes)*
        #visibility #signature {
            if [#(#versions),*].contains(&self.version) {
                #function_body
            } else {
                Err(crate::error::Error::UnsupportedOperation(self.version))
            }
        }
    )
    .into()
}
