use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_RUN, Value};

#[bolt_structure(SIGNATURE_RUN)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Run {
    pub(crate) statement: String,
    pub(crate) parameters: HashMap<String, Value>,
}

impl Run {
    pub fn new(statement: String, parameters: HashMap<String, Value>) -> Self {
        Self {
            statement,
            parameters,
        }
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }
}

impl_try_from_message!(Run, Run);
