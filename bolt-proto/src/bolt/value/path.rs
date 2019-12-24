use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::Value;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x50;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Path {
    nodes: Box<Value>,
    // TODO: The relationships property is a list of UnboundRelationship - make sure this works as expected
    relationships: Box<Value>,
    sequence: Box<Value>,
}

// TODO: impl From<[Native Path type]> for Node

impl TryFrom<Value> for Path {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Path(path) => Ok(path),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native Path type]> for Value
