use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bolt;
use crate::bolt::Message;
use crate::error::*;

#[derive(Debug)]
pub struct Init {
    pub(crate) client_name: String,
    pub(crate) auth_token: HashMap<String, String>,
}

impl Init {
    pub fn new(client_name: String, auth_token: HashMap<String, String>) -> Self {
        Self {
            client_name,
            auth_token,
        }
    }

    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    pub fn auth_token(&self) -> &HashMap<String, String> {
        &self.auth_token
    }
}

impl TryFrom<bolt::message::Init> for Init {
    type Error = Error;

    fn try_from(bolt_init: bolt::message::Init) -> Result<Self> {
        Ok(Init {
            client_name: String::try_from(bolt_init.client_name)?,
            auth_token: bolt_init.auth_token.try_into()?,
        })
    }
}

impl TryFrom<Message> for Init {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Init(init) => Ok(Init::try_from(init)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
