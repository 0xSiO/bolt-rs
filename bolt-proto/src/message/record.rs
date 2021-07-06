use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_RECORD, Value};

#[bolt_structure(SIGNATURE_RECORD)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Record {
    pub(crate) fields: Vec<Value>,
}

impl Record {
    pub fn new(fields: Vec<Value>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[Value] {
        &self.fields
    }
}

impl_try_from_message!(Record, Record);
