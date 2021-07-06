use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_FAILURE, Value};

#[bolt_structure(SIGNATURE_FAILURE)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Failure {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Failure);
impl_try_from_message!(Failure, Failure);
