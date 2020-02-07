use std::collections::HashMap;
use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x4E;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Node {
    pub(crate) node_identity: i64,
    pub(crate) labels: Vec<String>,
    pub(crate) properties: HashMap<String, Value>,
}

impl Node {
    pub fn new(
        node_identity: i64,
        labels: Vec<String>,
        properties: HashMap<String, impl Into<Value>>,
    ) -> Self {
        Self {
            node_identity,
            labels,
            properties: properties.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }

    pub fn node_identity(&self) -> i64 {
        self.node_identity
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.properties
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
