use std::collections::HashMap;

use crate::bolt::value::{BoltValue, Integer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    node_identity: Integer,
    labels: Vec<String>,
    properties: HashMap<String, BoltValue>,
}
