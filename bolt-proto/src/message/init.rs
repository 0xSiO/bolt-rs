use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_INIT, Value};

#[bolt_structure(SIGNATURE_INIT)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Init {
    pub(crate) client_name: String,
    pub(crate) auth_token: HashMap<String, Value>,
}

impl Init {
    pub fn new(client_name: String, auth_token: HashMap<String, Value>) -> Self {
        Self {
            client_name,
            auth_token,
        }
    }

    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    pub fn auth_token(&self) -> &HashMap<String, Value> {
        &self.auth_token
    }
}

impl_try_from_message!(Init, Init);
