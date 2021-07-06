use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_SUCCESS, Value};

#[bolt_structure(SIGNATURE_SUCCESS)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Success {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Success);
impl_try_from_message!(Success, Success);
