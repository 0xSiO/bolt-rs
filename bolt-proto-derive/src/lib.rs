#![warn(rust_2018_idioms)]

use proc_macro::TokenStream;

use syn::{AttributeArgs, Fields, Generics, Ident, ItemStruct, NestedMeta, WhereClause};

use quote::{format_ident, quote};

pub(crate) const MARKER_TINY_STRUCT: u8 = 0xB0;
pub(crate) const MARKER_SMALL_STRUCT: u8 = 0xDC;
pub(crate) const MARKER_MEDIUM_STRUCT: u8 = 0xDD;

fn get_struct_info(
    structure: ItemStruct,
    args: AttributeArgs,
) -> (Ident, Generics, Option<WhereClause>, Fields, u8, NestedMeta) {
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

    let signature = args.into_iter().next().expect("signature is required");

    (name, type_args, where_clause, fields, marker, signature)
}

#[proc_macro_attribute]
pub fn bolt_structure(attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let structure = syn::parse_macro_input!(item as ItemStruct);
    let args = syn::parse_macro_input!(attr_args as AttributeArgs);
    let (name, type_args, where_clause, fields, marker, signature) =
        get_struct_info(structure.clone(), args);

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
        .map(|name| quote!(#name: #name.try_into()?,));

    quote!(
        #structure

        impl#type_args crate::serialization::BoltValue for #name#type_args
        #where_clause
        {
            fn marker(&self) -> crate::error::SerializeResult<u8> {
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
