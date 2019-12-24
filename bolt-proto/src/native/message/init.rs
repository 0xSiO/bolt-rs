use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::message::Message;
use crate::bolt::value::Value;
use crate::error::MessageError;

#[derive(Debug)]
pub struct Init {
    pub client_name: String,
    pub auth_token: HashMap<String, Value>,
}

impl TryFrom<bolt::message::Init> for Init {
    type Error = Error;

    fn try_from(bolt_init: bolt::message::Init) -> Result<Self, Self::Error> {
        Ok(Init {
            client_name: String::try_from(bolt_init.client_name)?,
            auth_token: bolt_init.auth_token.try_into()?,
        })
    }
}

impl TryFrom<Message> for Init {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Init(init) => Ok(Init::try_from(init)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
