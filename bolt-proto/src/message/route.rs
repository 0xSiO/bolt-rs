use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, message::SIGNATURE_ROUTE, Value};

#[bolt_structure(SIGNATURE_ROUTE)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Route {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Route);
impl_try_from_message!(Route, Route);
