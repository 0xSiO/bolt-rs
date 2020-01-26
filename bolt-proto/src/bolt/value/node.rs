use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::bolt::Value;
use crate::error::*;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x4E;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Node {
    pub(crate) node_identity: Box<Value>,
    pub(crate) labels: Box<Value>,
    pub(crate) properties: Box<Value>,
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

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Node(node) => Ok(node),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
