use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x3F;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Pull {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Pull {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl_try_from_message!(Pull, Pull);

// TODO: Tests
