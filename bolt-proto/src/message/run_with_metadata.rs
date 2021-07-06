use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_RUN_WITH_METADATA, Value};

#[bolt_structure(SIGNATURE_RUN_WITH_METADATA)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunWithMetadata {
    pub(crate) statement: String,
    pub(crate) parameters: HashMap<String, Value>,
    pub(crate) metadata: HashMap<String, Value>,
}

impl RunWithMetadata {
    pub fn new(
        statement: String,
        parameters: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
    ) -> Self {
        Self {
            statement,
            parameters,
            metadata,
        }
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl_try_from_message!(RunWithMetadata, RunWithMetadata);
