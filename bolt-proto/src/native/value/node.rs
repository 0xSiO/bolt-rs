use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bolt;
use crate::bolt::Value;
use crate::error::Error;
use crate::error::ValueError;

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl TryFrom<bolt::value::Node> for Node {
    type Error = Error;

    fn try_from(bolt_node: bolt::value::Node) -> Result<Self, Self::Error> {
        Ok(Node {
            node_identity: i64::try_from(*bolt_node.node_identity)?,
            labels: (*bolt_node.labels).try_into()?,
            properties: (*bolt_node.properties).try_into()?,
        })
    }
}

impl TryFrom<Value> for Node {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Node(node) => Node::try_from(node),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
