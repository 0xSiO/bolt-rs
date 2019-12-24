use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::Value;
use crate::error::ValueError;
use crate::native;

pub const SIGNATURE: u8 = 0x50;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Path {
    pub nodes: Box<Value>,
    // TODO: The relationships property is a list of UnboundRelationship - make sure this works as expected
    pub relationships: Box<Value>,
    pub sequence: Box<Value>,
}

impl From<native::value::Path> for Path {
    fn from(native_path: native::value::Path) -> Self {
        Self {
            nodes: Box::new(Value::from(native_path.nodes)),
            relationships: Box::new(Value::from(native_path.relationships)),
            sequence: Box::new(Value::from(native_path.sequence)),
        }
    }
}

impl TryFrom<Value> for Path {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Path(path) => Ok(path),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
