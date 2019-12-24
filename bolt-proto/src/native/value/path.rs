use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::value::Value;
use crate::error::ValueError;
use crate::native::value::node::Node;
use crate::native::value::relationship::Relationship;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Path {
    pub nodes: Vec<Node>,
    pub relationships: Vec<Relationship>,
    pub sequence: i64,
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
