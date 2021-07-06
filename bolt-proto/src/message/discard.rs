use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_DISCARD, Value};

#[bolt_structure(SIGNATURE_DISCARD)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Discard {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Discard);
impl_try_from_message!(Discard, Discard);
