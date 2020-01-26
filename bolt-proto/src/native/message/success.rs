use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bolt;
use crate::bolt::Message;
use crate::bolt::Value;
use crate::error::Error;
use crate::error::MessageError;

#[derive(Debug)]
pub struct Success {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Success {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl TryFrom<bolt::message::Success> for Success {
    type Error = Error;

    fn try_from(bolt_success: bolt::message::Success) -> Result<Self, Self::Error> {
        Ok(Success {
            metadata: bolt_success.metadata.try_into()?,
        })
    }
}

impl TryFrom<Message> for Success {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Success(success) => Ok(Success::try_from(success)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
