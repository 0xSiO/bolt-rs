use std::collections::HashMap;

use crate::bolt::value::{BoltValue, Integer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Relationship {
    rel_identity: Integer,
    start_node_identity: Integer,
    end_node_identity: Integer,
    rel_type: String,
    properties: HashMap<String, BoltValue>,
}
