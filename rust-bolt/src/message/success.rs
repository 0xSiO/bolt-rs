use rust_bolt_macros::*;

use crate::serialize::Serialize;
use crate::structure::Structure;
use crate::value::{Map, String, Value};

#[derive(Debug, Structure, Marker, Serialize)]
pub struct Success {
    metadata: Map<String, Value>,
}
