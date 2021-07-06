use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_PULL, Value};

#[bolt_structure(SIGNATURE_PULL)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pull {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Pull);
impl_try_from_message!(Pull, Pull);
