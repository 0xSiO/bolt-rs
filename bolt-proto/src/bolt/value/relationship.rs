use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::Value;
use crate::error::ValueError;
use crate::native;

pub const SIGNATURE: u8 = 0x52;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Relationship {
    pub rel_identity: Box<Value>,
    pub start_node_identity: Box<Value>,
    pub end_node_identity: Box<Value>,
    pub rel_type: Box<Value>,
    pub properties: Box<Value>,
}

impl From<native::value::Relationship> for Relationship {
    fn from(native_rel: native::value::Relationship) -> Self {
        Self {
            rel_identity: Box::new(Value::from(native_rel.rel_identity)),
            start_node_identity: Box::new(Value::from(native_rel.start_node_identity)),
            end_node_identity: Box::new(Value::from(native_rel.end_node_identity)),
            rel_type: Box::new(Value::from(native_rel.rel_type)),
            properties: Box::new(Value::from(native_rel.properties)),
        }
    }
}

impl TryFrom<Value> for Relationship {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Relationship(rel) => Ok(rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
