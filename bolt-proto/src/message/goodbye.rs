use bolt_proto_derive::*;

pub(crate) const MARKER: u8 = 0xB0;
pub(crate) const SIGNATURE: u8 = 0x02;

#[derive(Debug, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Goodbye;

#[cfg(test)]
crate::impl_empty_message_tests!(Goodbye);
