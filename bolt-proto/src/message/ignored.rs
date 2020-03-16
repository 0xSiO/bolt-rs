use bolt_proto_derive::*;

pub(crate) const MARKER: u8 = 0xB0;
pub(crate) const SIGNATURE: u8 = 0x7E;

#[derive(Debug, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Ignored;

#[cfg(test)]
crate::impl_empty_message_tests!(Ignored);
