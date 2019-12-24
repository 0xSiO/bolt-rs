use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::Value;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x72;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct UnboundRelationship {
    rel_identity: Box<Value>,
    rel_type: Box<Value>,
    properties: Box<Value>,
}

// TODO: impl From<[Native UnboundRelationship type]> for Node

impl TryFrom<Value> for UnboundRelationship {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::UnboundRelationship(unbound_rel) => Ok(unbound_rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native UnboundRelationship type]> for Value
