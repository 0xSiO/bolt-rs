use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, message::SIGNATURE_INIT, Value};

#[bolt_structure(SIGNATURE_INIT)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Init {
    pub(crate) user_agent: String,
    pub(crate) auth_token: HashMap<String, Value>,
}

impl Init {
    pub fn new(user_agent: String, auth_token: HashMap<String, Value>) -> Self {
        Self {
            user_agent,
            auth_token,
        }
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn auth_token(&self) -> &HashMap<String, Value> {
        &self.auth_token
    }
}

impl_try_from_message!(Init, Init);
