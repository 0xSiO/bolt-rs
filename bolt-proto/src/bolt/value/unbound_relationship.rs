use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::BoltValue;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x72;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct UnboundRelationship {
    rel_identity: Box<BoltValue>,
    rel_type: Box<BoltValue>,
    properties: Box<BoltValue>,
}

// TODO: impl From<[Native UnboundRelationship type]> for Node

impl TryFrom<BoltValue> for UnboundRelationship {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::UnboundRelationship(unbound_rel) => Ok(unbound_rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native UnboundRelationship type]> for BoltValue
