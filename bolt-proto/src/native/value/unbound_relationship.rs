use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::Value;
use crate::error::ValueError;

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl TryFrom<bolt::value::UnboundRelationship> for UnboundRelationship {
    type Error = Error;

    fn try_from(bolt_ub_rel: bolt::value::UnboundRelationship) -> Result<Self, Self::Error> {
        Ok(UnboundRelationship {
            rel_identity: i64::try_from(*bolt_ub_rel.rel_identity)?,
            rel_type: String::try_from(*bolt_ub_rel.rel_type)?,
            properties: (*bolt_ub_rel.properties).try_into()?,
        })
    }
}

impl TryFrom<Value> for UnboundRelationship {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::UnboundRelationship(ub_rel) => Ok(UnboundRelationship::try_from(ub_rel)?),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
