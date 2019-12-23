use std::convert::TryFrom;

use failure::Error;

use rust_bolt_macros::*;

use crate::bolt::value::BoltValue;
use crate::error::ValueError;

pub const SIGNATURE: u8 = 0x50;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Path {
    nodes: Box<BoltValue>,
    // TODO: The relationships property is a list of UnboundRelationship - make sure this works as expected
    relationships: Box<BoltValue>,
    sequence: Box<BoltValue>,
}

// TODO: impl From<[Native Path type]> for Node

impl TryFrom<BoltValue> for Path {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::Path(path) => Ok(path),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

// TODO: impl From<[Native Path type]> for BoltValue
