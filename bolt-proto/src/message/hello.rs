use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_HELLO, Value};

#[bolt_structure(SIGNATURE_HELLO)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Hello {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Hello);
impl_try_from_message!(Hello, Hello);
