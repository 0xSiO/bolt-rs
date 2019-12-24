use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::value::Value;
use crate::error::ValueError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    pub node_identity: i64,
    pub labels: Vec<String>,
    pub properties: HashMap<String, Value>,
}

//impl TryFrom<bolt::value::Node> for Node {
//    type Error = Error;
//
//    fn try_from(bolt_node: bolt::value::Node) -> Result<Self, Self::Error> {
//        Ok(Node {
//            node_identity: i64::try_from(*bolt_node.node_identity)?,
//            labels: (*bolt_node.labels).try_into()?,
//            properties: (*bolt_node.properties).try_into()?,
//        })
//    }
//}
//
//impl TryFrom<Value> for Node {
//    type Error = Error;
//
//    fn try_from(value: Value) -> Result<Self, Self::Error> {
//        match value {
//            Value::Node(node) => Ok(Node::try_from(node)?),
//            _ => Err(ValueError::InvalidConversion(value).into()),
//        }
//    }
//}
