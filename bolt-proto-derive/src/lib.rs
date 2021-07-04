#![warn(rust_2018_idioms)]

use proc_macro::TokenStream;

use syn::{
    AttributeArgs, Data, DataStruct, Fields, Generics, Ident, ItemStruct, Lit, NestedMeta,
    WhereClause,
};

use quote::{format_ident, quote};

pub(crate) const MARKER_TINY_STRUCT: u8 = 0xB0;
pub(crate) const MARKER_SMALL_STRUCT: u8 = 0xDC;
pub(crate) const MARKER_MEDIUM_STRUCT: u8 = 0xDD;

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

fn get_struct_info_new(
    structure: ItemStruct,
    args: AttributeArgs,
) -> (Ident, Generics, Option<WhereClause>, Fields, u8, u8) {
    let name = structure.ident;
    let type_args = structure.generics;
    let where_clause = type_args.where_clause.clone();
    let fields = structure.fields;

    let marker = match fields.len() {
        0..=15 => MARKER_TINY_STRUCT | fields.len() as u8,
        16..=255 => MARKER_SMALL_STRUCT,
        256..=65535 => MARKER_MEDIUM_STRUCT,
        _ => panic!("struct has too many fields"),
    };

    let signature = match args.into_iter().next().expect("signature is required") {
        NestedMeta::Lit(Lit::Int(int)) => int
            .base10_parse::<u8>()
            .expect("couldn't parse signature byte"),
        other => panic!("invalid signature byte: {:?}", other),
    };

    (name, type_args, where_clause, fields, marker, signature)
}

#[proc_macro_attribute]
pub fn bolt_structure(attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let structure = syn::parse_macro_input!(item as ItemStruct);
    let args = syn::parse_macro_input!(attr_args as AttributeArgs);
    let (name, type_args, where_clause, fields, marker, signature) =
        get_struct_info_new(structure.clone(), args);

    let field_names: Vec<Ident> = fields.into_iter().map(|f| f.ident.unwrap()).collect();
    let byte_var_names: Vec<Ident> = field_names
        .iter()
        .map(|name| format_ident!("{}_bytes", name))
        .collect();

    let byte_var_defs = byte_var_names.iter()
        .zip(field_names.iter())
        .map(|(var_name, field_name)| {
            quote!(let #var_name = crate::Value::from(self.#field_name).serialize()?;)
        });

    let deserialize_var_defs = field_names.iter().map(|name| {
        quote!(
            let (#name, remaining) = crate::Value::deserialize(bytes)?;
            bytes = remaining;
        )
    });

    let deserialize_fields = field_names
        .iter()
        // TODO: Replace unwrap() with ?, after changing the error type to ConversionError
        .map(|name| quote!(#name: #name.try_into().unwrap(),));

    quote!(
        #structure

        impl#type_args crate::serialization::BoltValue for #name#type_args
        #where_clause
        {
            fn marker(&self) -> crate::error::MarkerResult<u8> {
                Ok(#marker)
            }

            fn serialize(self) -> crate::error::SerializeResult<::bytes::Bytes> {
                use ::bytes::BufMut;
                use crate::serialization::{BoltStructure, BoltValue};

                let marker = self.marker()?;
                let signature = self.signature();
                #(#byte_var_defs)*

                // Marker byte, signature byte, then the rest of the data
                let mut result_bytes_mut = ::bytes::BytesMut::with_capacity(
                    std::mem::size_of::<u8>() * 2 #(+ #byte_var_names.len())*
                );
                result_bytes_mut.put_u8(marker);
                result_bytes_mut.put_u8(signature);
                #(result_bytes_mut.put(#byte_var_names);)*
                Ok(result_bytes_mut.freeze())
            }

            fn deserialize<B>(mut bytes: B) -> crate::error::DeserializeResult<(Self, B)>
            where B: ::bytes::Buf + ::std::panic::UnwindSafe
            {
                use ::std::convert::TryInto;
                #(#deserialize_var_defs)*
                Ok((Self { #(#deserialize_fields)* }, bytes))
            }
        }

        impl#type_args crate::serialization::BoltStructure for #name#type_args
        #where_clause
        {
            fn signature(&self) -> u8 {
                #signature
            }
        }
    )
    .into()
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
