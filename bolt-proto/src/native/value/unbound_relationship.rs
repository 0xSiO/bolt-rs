use std::collections::HashMap;

use crate::bolt::value::{Integer, Value};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnboundRelationship {
    rel_identity: Integer,
    rel_type: String,
    properties: HashMap<String, Value>,
}
