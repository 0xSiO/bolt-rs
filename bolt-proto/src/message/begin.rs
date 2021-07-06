use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_BEGIN, Value};

#[bolt_structure(SIGNATURE_BEGIN)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Begin {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Begin);
impl_try_from_message!(Begin, Begin);
