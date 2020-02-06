use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::v1::error::*;
use crate::v1::value::*;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x50;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
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

impl TryFrom<Value> for Path {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Path(path) => Ok(Path::try_from(path)?),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
