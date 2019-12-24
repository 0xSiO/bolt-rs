use crate::bolt::value::Integer;
use crate::native::value::node::Node;
use crate::native::value::relationship::Relationship;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Path {
    nodes: Vec<Node>,
    relationships: Vec<Relationship>,
    sequence: Integer,
}
