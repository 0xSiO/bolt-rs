use std::convert::{TryFrom, TryInto};

use crate::bolt;
use crate::bolt::Value;
use crate::error::*;
use crate::value::{Node, UnboundRelationship};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Path {
    pub(crate) nodes: Vec<Node>,
    pub(crate) relationships: Vec<UnboundRelationship>,
    pub(crate) sequence: i64,
}

impl Path {
    pub fn new(nodes: Vec<Node>, relationships: Vec<UnboundRelationship>, sequence: i64) -> Self {
        Self {
            nodes,
            relationships,
            sequence,
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn relationships(&self) -> &[UnboundRelationship] {
        &self.relationships
    }

    pub fn sequence(&self) -> i64 {
        self.sequence
    }
}

impl TryFrom<bolt::value::Path> for Path {
    type Error = Error;

    fn try_from(bolt_path: bolt::value::Path) -> Result<Self> {
        Ok(Path {
            nodes: (*bolt_path.nodes).try_into()?,
            relationships: (*bolt_path.relationships).try_into()?,
            sequence: i64::try_from(*bolt_path.sequence)?,
        })
    }
}

impl TryFrom<Value> for Path {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Path(path) => Ok(Path::try_from(path)?),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
