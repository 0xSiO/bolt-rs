use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::Value;
use crate::error::ValueError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Path {
    pub(crate) nodes: Vec<Value>,
    pub(crate) relationships: Vec<Value>,
    pub(crate) sequence: i64,
}

impl Path {
    pub fn new(
        nodes: Vec<impl Into<Value>>,
        relationships: Vec<impl Into<Value>>,
        sequence: i64,
    ) -> Self {
        Self {
            nodes: nodes.into_iter().map(|v| v.into()).collect(),
            relationships: relationships.into_iter().map(|v| v.into()).collect(),
            sequence,
        }
    }
}

impl TryFrom<bolt::value::Path> for Path {
    type Error = Error;

    fn try_from(bolt_path: bolt::value::Path) -> Result<Self, Self::Error> {
        Ok(Path {
            nodes: (*bolt_path.nodes).try_into()?,
            relationships: (*bolt_path.relationships).try_into()?,
            sequence: i64::try_from(*bolt_path.sequence)?,
        })
    }
}

impl TryFrom<Value> for Path {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Path(path) => Ok(Path::try_from(path)?),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
