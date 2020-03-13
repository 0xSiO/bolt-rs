use std::collections::HashMap;
use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x72;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct UnboundRelationship {
    pub(crate) rel_identity: i64,
    pub(crate) rel_type: String,
    pub(crate) properties: HashMap<String, Value>,
}

impl UnboundRelationship {
    pub fn new(
        rel_identity: i64,
        rel_type: String,
        properties: HashMap<String, impl Into<Value>>,
    ) -> Self {
        Self {
            rel_identity,
            rel_type,
            properties: properties.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }

    pub fn rel_identity(&self) -> i64 {
        self.rel_identity
    }

    pub fn rel_type(&self) -> &str {
        &self.rel_type
    }

    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.properties
    }
}

impl TryFrom<Value> for UnboundRelationship {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::UnboundRelationship(ub_rel) => Ok(ub_rel),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
