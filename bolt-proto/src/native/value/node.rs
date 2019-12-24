use std::collections::HashMap;

use crate::bolt::value::{Integer, Value};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    node_identity: Integer,
    labels: Vec<String>,
    properties: HashMap<String, Value>,
}
