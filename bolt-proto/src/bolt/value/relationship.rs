use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::BoltValue;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x52;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Relationship {
    rel_identity: Box<BoltValue>,
    start_node_identity: Box<BoltValue>,
    end_node_identity: Box<BoltValue>,
    rel_type: Box<BoltValue>,
    properties: Box<BoltValue>,
}

// TODO: impl From<[Native Relationship type]> for Node

impl TryFrom<BoltValue> for Relationship {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::Relationship(rel) => Ok(rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native Relationship type]> for BoltValue
