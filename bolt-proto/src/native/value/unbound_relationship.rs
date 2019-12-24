use std::collections::HashMap;

use crate::bolt::value::{BoltValue, Integer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnboundRelationship {
    rel_identity: Integer,
    rel_type: String,
    properties: HashMap<String, BoltValue>,
}
