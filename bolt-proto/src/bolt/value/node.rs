use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::value::Value;
use crate::error::ValueError;
use crate::native;

pub const SIGNATURE: u8 = 0x4E;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Node {
    pub node_identity: Box<Value>,
    pub labels: Box<Value>,
    pub properties: Box<Value>,
}

impl From<native::value::Node> for Node {
    fn from(native_node: native::value::Node) -> Self {
        Self {
            node_identity: Box::new(Value::from(native_node.node_identity)),
            labels: Box::new(Value::from(native_node.labels)),
            properties: Box::new(Value::from(native_node.properties)),
        }
    }
}

impl TryFrom<Value> for Node {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Node(node) => Ok(node),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
