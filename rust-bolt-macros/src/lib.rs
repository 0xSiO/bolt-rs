extern crate proc_macro;

use proc_macro::TokenStream;

use syn::{Data, DataStruct, Ident};

use quote::{format_ident, quote};

#[proc_macro_derive(Structure)]
pub fn structure_derive(input: TokenStream) -> TokenStream {
    impl_structure(&syn::parse(input).unwrap())
}

fn impl_structure(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let type_args = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields = match &ast.data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("Macro must be used on a struct."),
    };
    let marker = get_structure_marker(fields.len());
    let name_str = name.to_string();
    let signature = get_structure_signature(&name_str);

    let byte_var_names: Vec<Ident> = fields
        .iter()
        .map(|f| format_ident!("{}_bytes", f.ident.clone().unwrap()))
        .collect();

    let byte_vars = byte_var_names.iter().zip(fields).map(|(var_name, field)| {
        let field_name = field.ident.clone();
        quote!(let #var_name = self.#field_name.try_into_bytes()?;)
    });

    let size_bytes = get_size_bytes(fields.len());

    let gen = quote! {
        use ::std::convert::TryInto;
        use ::bytes::BufMut;
        use crate::value::Marker;

        impl#type_args crate::structure::Structure for #name#type_args
        #where_clause
        {
            fn get_signature(&self) -> u8 {
                #signature
            }
        }

        impl#type_args crate::value::Marker for #name#type_args
        #where_clause
        {
            fn get_marker(&self) -> Result<u8, ::failure::Error> {
                Ok(#marker)
            }
        }

        impl#type_args crate::serialize::Serialize for #name#type_args
        #where_clause
        {}

        impl#type_args TryInto<::bytes::Bytes> for #name#type_args
        #where_clause
        {
            type Error = ::failure::Error;

            fn try_into(self) -> Result<::bytes::Bytes, Self::Error> {
                let marker = self.get_marker()?;
                let signature = self.get_signature();
                #(#byte_vars)*
                // Marker byte, up to 2 size bytes, signature byte, then the rest of the data
                let mut result_bytes_mut = ::bytes::BytesMut::with_capacity(
                    std::mem::size_of::<u8>() * 4 #(+ #byte_var_names.len())*
                );
                result_bytes_mut.put_u8(marker);
                #(result_bytes_mut.put_u8(#size_bytes);)*
                result_bytes_mut.put_u8(signature);
                #(result_bytes_mut.put(#byte_var_names);)*
                Ok(result_bytes_mut.freeze())
            }
        }
    };
    gen.into()
}

fn get_structure_signature(struct_name: &str) -> u8 {
    match struct_name {
        "BoltInit" => 0x01,
        "BoltSuccess" => 0x70,
        _ => panic!("Invalid message type: {}", struct_name),
    }
}

const MARKER_TINY_STRUCTURE: u8 = 0xB0;
const MARKER_SMALL_STRUCTURE: u8 = 0xDC;
const MARKER_MEDIUM_STRUCTURE: u8 = 0xDD;

fn get_structure_marker(num_fields: usize) -> u8 {
    match num_fields {
        0..=15 => MARKER_TINY_STRUCTURE | num_fields as u8,
        16..=255 => MARKER_SMALL_STRUCTURE,
        256..=65_535 => MARKER_MEDIUM_STRUCTURE,
        _ => panic!("Too many fields in struct"),
    }
}

fn get_size_bytes(num_fields: usize) -> Vec<u8> {
    match num_fields {
        0..=15 => vec![],
        16..=255 => (num_fields as u8).to_be_bytes().to_vec(),
        256..=65_535 => (num_fields as u16).to_be_bytes().to_vec(),
        _ => panic!("Too many fields in struct"),
    }
}
