use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::impl_try_from_value;
use crate::value::String;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB5;
pub(crate) const SIGNATURE: u8 = 0x52;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Relationship {
    pub(crate) rel_identity: i64,
    pub(crate) start_node_identity: i64,
    pub(crate) end_node_identity: i64,
    pub(crate) rel_type: String,
    pub(crate) properties: HashMap<String, Value>,
}

impl Relationship {
    pub fn new(
        rel_identity: i64,
        start_node_identity: i64,
        end_node_identity: i64,
        rel_type: std::string::String,
        properties: HashMap<std::string::String, impl Into<Value>>,
    ) -> Self {
        Self {
            rel_identity,
            start_node_identity,
            end_node_identity,
            rel_type: rel_type.into(),
            properties: properties
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    pub fn rel_identity(&self) -> i64 {
        self.rel_identity
    }

    pub fn start_node_identity(&self) -> i64 {
        self.start_node_identity
    }

    pub fn end_node_identity(&self) -> i64 {
        self.end_node_identity
    }

    pub fn rel_type(&self) -> &str {
        &self.rel_type.value
    }

    pub fn properties(&self) -> HashMap<&str, &Value> {
        self.properties
            .iter()
            .map(|(k, v)| (k.value.as_str(), v))
            .collect()
    }
}

impl_try_from_value!(Relationship, Relationship);
