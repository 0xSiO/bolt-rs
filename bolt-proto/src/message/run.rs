use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_RUN, Value};

#[bolt_structure(SIGNATURE_RUN)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Run {
    pub(crate) query: String,
    pub(crate) parameters: HashMap<String, Value>,
}

impl Run {
    pub fn new(query: String, parameters: HashMap<String, Value>) -> Self {
        Self { query, parameters }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }
}

impl_try_from_message!(Run, Run);
