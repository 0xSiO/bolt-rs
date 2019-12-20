extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

#[proc_macro_derive(Structure)]
pub fn structure_derive(input: TokenStream) -> TokenStream {
    impl_structure(&syn::parse(input).unwrap())
}

fn impl_structure(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let type_args = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let name_str = name.to_string();
    let signature: u8 = match name_str.as_str() {
        "Init" => 0x01,
        _ => panic!("Invalid message type: {}", name_str),
    };

    let gen = quote! {
        impl#type_args crate::structure::Structure for #name#type_args
        #where_clause
        {
            fn get_signature(&self) -> u8 {
                #signature
            }
        }
    };
    gen.into()
}
