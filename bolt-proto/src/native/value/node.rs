use std::collections::HashMap;

use crate::bolt::value::Value;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    pub node_identity: i64,
    pub labels: Vec<String>,
    pub properties: HashMap<String, Value>,
}

// TODO: Running into issues converting a Value to native type (Vec and HashMap)
//       Make sure String implements TryFrom<Value>.
//impl TryFrom<bolt::value::Node> for Node {
//    type Error = Error;
//
//    fn try_from(bolt_node: bolt::value::Node) -> Result<Self, Self::Error> {
//        Ok(Node {
//            node_identity: i64::try_from(*bolt_node.node_identity)?,
//            labels: Vec::try_from(*bolt_node.labels)?,
//            properties: HashMap::try_from(*bolt_node.properties)?,
//        })
//    }
//}

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
