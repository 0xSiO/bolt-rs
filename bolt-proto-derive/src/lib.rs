#![warn(rust_2018_idioms)]

use proc_macro::TokenStream;

use syn::{Data, DataStruct, Fields, Generics, Ident, WhereClause};

use quote::{format_ident, quote};

fn get_struct_info(ast: &syn::DeriveInput) -> (&Ident, &Generics, &Option<WhereClause>, &Fields) {
    let name = &ast.ident;
    let type_args = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields = match &ast.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("macro must be used on a struct."),
    };
    (name, type_args, where_clause, fields)
}

#[proc_macro_derive(Signature)]
pub fn signature_derive(input: TokenStream) -> TokenStream {
    let ast = &syn::parse(input).unwrap();
    let (name, type_args, where_clause, _fields) = get_struct_info(ast);

    quote!(
        impl#type_args crate::serialization::Signature for #name#type_args
        #where_clause
        {
            fn get_signature(&self) -> u8 {
                SIGNATURE
            }
        }
    )
    .into()
}

#[proc_macro_derive(Marker)]
pub fn marker_derive(input: TokenStream) -> TokenStream {
    let ast = &syn::parse(input).unwrap();
    let (name, type_args, where_clause, _fields) = get_struct_info(ast);
    quote!(
        impl#type_args crate::serialization::Marker for #name#type_args
        #where_clause
        {
            fn get_marker(&self) -> crate::error::Result<u8> {
                Ok(MARKER)
            }
        }
    )
    .into()
}

#[proc_macro_derive(Serialize)]
pub fn serialize_derive(input: TokenStream) -> TokenStream {
    let ast = &syn::parse(input).unwrap();
    let (name, type_args, where_clause, fields) = get_struct_info(ast);

    let byte_var_names: Vec<Ident> = fields
        .iter()
        .map(|f| format_ident!("{}_bytes", f.ident.clone().unwrap()))
        .collect();

    let byte_vars = byte_var_names.iter().zip(fields).map(|(var_name, field)| {
        let field_name = field.ident.clone();
        quote!(let #var_name = crate::Value::from(self.#field_name).try_into_bytes()?;)
    });

    quote!(
        impl#type_args crate::serialization::Serialize for #name#type_args
        #where_clause
        {}

        impl#type_args ::std::convert::TryInto<::bytes::Bytes> for #name#type_args
        #where_clause
        {
            type Error = crate::error::Error;

            fn try_into(self) -> crate::error::Result<::bytes::Bytes> {
                use ::std::convert::{TryFrom, TryInto};
                use ::bytes::BufMut;
                use crate::serialization::Serialize;

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
    )
    .into()
}

#[proc_macro_derive(Deserialize)]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let ast = &syn::parse(input).unwrap();
    let (name, type_args, where_clause, fields) = get_struct_info(ast);

    let deserialize_fields =
        fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote!(#field_name: crate::Value::try_from(::std::sync::Arc::clone(&remaining_bytes_arc))?.try_into()?,)
            });

    quote!(
        impl crate::serialization::Deserialize for #name#type_args
        #where_clause
        {}

        impl ::std::convert::TryFrom<::std::sync::Arc<::std::sync::Mutex<::bytes::Bytes>>> for #name#type_args
        #where_clause
        {
            type Error = crate::error::Error;

            fn try_from(remaining_bytes_arc: ::std::sync::Arc<::std::sync::Mutex<::bytes::Bytes>>) -> crate::error::Result<Self> {
                use ::std::convert::{TryFrom, TryInto};
                Ok(#name {
                    #(#deserialize_fields)*
                })
            }
        }
    ).into()
}
