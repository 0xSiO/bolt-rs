extern crate proc_macro;

use proc_macro::TokenStream;

use syn::{Data, DataStruct, GenericArgument, Ident, PathArguments, Type, TypePath};

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
    let marker = get_structure_marker(fields.len());

    let byte_var_names: Vec<Ident> = fields
        .iter()
        .map(|f| format_ident!("{}_bytes", f.ident.clone().unwrap()))
        .collect();

    let byte_vars = byte_var_names.iter().zip(fields).map(|(var_name, field)| {
        let field_name = field.ident.clone();
        quote!(let #var_name = self.#field_name.try_into_bytes()?;)
    });

    let deserialize_fields =
        fields
            .iter()
            .map(|field| match (&field.ident.clone().unwrap(), field.ty.clone()) {
                (field_name, Type::Path(TypePath { path, .. })) => {
                    let types: Vec<String> = path.segments.iter().map(|s| {
                        let type_args = match &s.arguments {
                            PathArguments::AngleBracketed(args) => {
                                match &args.args[0] {
                                    GenericArgument::Type(Type::Path(TypePath { path, .. })) => {
                                        format!("<{}>", path.segments[0].ident)
                                    }
                                    arg => panic!("Can't derive Deserialize for type argument {:?} on field {}", arg, field_name)
                                }
                            }
                            _ => "".to_string(),
                        };
                        s.ident.to_string() + &type_args
                    }).collect();
                    match types[0].as_str() {
                        "Box<BoltValue>" => {
                            quote!(#field_name: Box::new(BoltValue::try_from(::std::sync::Arc::clone(&remaining_bytes_arc))?),)
                        }
                        "BoltValue" => {
                            quote!(#field_name: BoltValue::try_from(::std::sync::Arc::clone(&remaining_bytes_arc))?,)
                        }
                        other => panic!("Can't deserialize {} with type {}", field_name, other),
                    }
                }
                _ => unreachable!(),
            });

    let size_bytes = get_size_bytes(fields.len());

    let gen = quote! {
        use ::bytes::BufMut;
        use crate::bolt::value::Marker;
        use crate::bolt::structure::Signature;
        use crate::serialize::Serialize;

        impl#type_args crate::bolt::structure::Signature for #name#type_args
        #where_clause
        {
            fn get_signature(&self) -> u8 {
                SIGNATURE
            }
        }

        impl#type_args crate::bolt::value::Marker for #name#type_args
        #where_clause
        {
            fn get_marker(&self) -> Result<u8, ::failure::Error> {
                Ok(#marker)
            }
        }

        impl#type_args crate::serialize::Serialize for #name#type_args
        #where_clause
        {}

        impl#type_args ::std::convert::TryInto<::bytes::Bytes> for #name#type_args
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

        impl crate::serialize::Deserialize for #name#type_args
        #where_clause
        {}

        impl ::std::convert::TryFrom<::std::sync::Arc<::std::sync::Mutex<::bytes::Bytes>>> for #name#type_args
        #where_clause
        {
            type Error = ::failure::Error;

            fn try_from(remaining_bytes_arc: ::std::sync::Arc<::std::sync::Mutex<::bytes::Bytes>>) -> Result<Self, Self::Error> {
                Ok(#name {
                    #(#deserialize_fields)*
                })
            }
        }
    };
    gen.into()
}

// Copied from structure module in bolt_proto
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
