use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::BoltValue;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x4E;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Node {
    node_identity: Box<BoltValue>,
    labels: Box<BoltValue>,
    properties: Box<BoltValue>,
}

// TODO: impl From<[Native Node type]> for Node

impl TryFrom<BoltValue> for Node {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::Node(node) => Ok(node),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native Node type]> for BoltValue
