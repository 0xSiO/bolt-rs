use bolt_proto_derive::*;

use crate::value::{Node, UnboundRelationship, SIGNATURE_PATH};

#[bolt_structure(SIGNATURE_PATH)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Path {
    pub(crate) nodes: Vec<Node>,
    pub(crate) relationships: Vec<UnboundRelationship>,
    pub(crate) sequence: Vec<i64>,
}

impl Path {
    pub fn new(
        nodes: Vec<Node>,
        relationships: Vec<UnboundRelationship>,
        sequence: Vec<i64>,
    ) -> Self {
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

    pub fn sequence(&self) -> &[i64] {
        &self.sequence
    }
}
