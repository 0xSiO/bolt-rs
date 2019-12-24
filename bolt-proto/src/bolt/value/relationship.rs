use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::Value;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x52;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Relationship {
    rel_identity: Box<Value>,
    start_node_identity: Box<Value>,
    end_node_identity: Box<Value>,
    rel_type: Box<Value>,
    properties: Box<Value>,
}

// TODO: impl From<[Native Relationship type]> for Node

impl TryFrom<Value> for Relationship {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Relationship(rel) => Ok(rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native Relationship type]> for Value
