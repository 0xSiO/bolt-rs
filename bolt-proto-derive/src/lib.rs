extern crate proc_macro;

use proc_macro::TokenStream;

use syn::{Data, DataStruct, Ident};

use quote::{format_ident, quote};

#[proc_macro_derive(Signature)]
pub fn signature_derive(input: TokenStream) -> TokenStream {
    impl_signature(&syn::parse(input).unwrap())
}

// I am so lazy, I just put all the impls into the Signature derive and made the other derives do nothing. This is so
// message structures can derive from all the traits and I only need to do the parsing and macro stuff once. Yes, I
// am aware this is a terrible thing to do.
// ------------------------------------------------------------------------------------------------------------------
#[proc_macro_derive(Marker)]
pub fn marker_derive(_input: TokenStream) -> TokenStream {
    quote!().into()
}

#[proc_macro_derive(Serialize)]
pub fn serialize_derive(_input: TokenStream) -> TokenStream {
    quote!().into()
}

#[proc_macro_derive(Deserialize)]
pub fn deserialize_derive(_input: TokenStream) -> TokenStream {
    quote!().into()
}
// ------------------------------------------------------------------------------------------------------------------

fn impl_signature(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let type_args = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields = match &ast.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("Macro must be used on a struct."),
    };

    let byte_var_names: Vec<Ident> = fields
        .iter()
        .map(|f| format_ident!("{}_bytes", f.ident.clone().unwrap()))
        .collect();

    let byte_vars = byte_var_names.iter().zip(fields).map(|(var_name, field)| {
        let field_name = field.ident.clone();
        quote!(let #var_name = crate::Value::from(self.#field_name).try_into_bytes()?;)
    });

    let deserialize_fields =
        fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote!(#field_name: crate::Value::try_from(::std::sync::Arc::clone(&remaining_bytes_arc))?.try_into()?,)
            });

    let gen = quote! {
        use ::bytes::BufMut;

        impl#type_args crate::Signature for #name#type_args
        #where_clause
        {
            fn get_signature(&self) -> u8 {
                SIGNATURE
            }
        }

        impl#type_args crate::Marker for #name#type_args
        #where_clause
        {
            fn get_marker(&self) -> crate::error::Result<u8> {
                Ok(MARKER)
            }
        }

        impl#type_args crate::Serialize for #name#type_args
        #where_clause
        {}

        impl#type_args ::std::convert::TryInto<::bytes::Bytes> for #name#type_args
        #where_clause
        {
            type Error = crate::error::Error;

            fn try_into(self) -> crate::error::Result<::bytes::Bytes> {
                use crate::serialize::Serialize;
                use ::std::convert::{TryFrom, TryInto};

                let marker = MARKER;
                let signature = SIGNATURE;
                #(#byte_vars)*
                // Marker byte, signature byte, then the rest of the data
                let mut result_bytes_mut = ::bytes::BytesMut::with_capacity(
                    std::mem::size_of::<u8>() * 2 #(+ #byte_var_names.len())*
                );
                result_bytes_mut.put_u8(MARKER);
                result_bytes_mut.put_u8(SIGNATURE);
                #(result_bytes_mut.put(#byte_var_names);)*
                Ok(result_bytes_mut.freeze())
            }
        }

        impl crate::Deserialize for #name#type_args
        #where_clause
        {}

        impl ::std::convert::TryFrom<::std::sync::Arc<::std::sync::Mutex<::bytes::Bytes>>> for #name#type_args
        #where_clause
        {
            type Error = ::failure::Error;

            fn try_from(remaining_bytes_arc: ::std::sync::Arc<::std::sync::Mutex<::bytes::Bytes>>) -> crate::error::Result<Self> {
                use ::std::convert::{TryFrom, TryInto};
                Ok(#name {
                    #(#deserialize_fields)*
                })
            }
        }
    };
    gen.into()
}
