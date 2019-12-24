use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::value::Value;
use crate::error::ValueError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnboundRelationship {
    pub rel_identity: i64,
    pub rel_type: String,
    pub properties: HashMap<String, Value>,
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
