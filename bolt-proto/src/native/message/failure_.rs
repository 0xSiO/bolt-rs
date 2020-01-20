use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt;
use crate::bolt::Message;
use crate::bolt::Value;
use crate::error::MessageError;

#[derive(Debug)]
pub struct Failure {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Failure {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl TryFrom<bolt::message::Failure> for Failure {
    type Error = Error;

    fn try_from(bolt_failure: bolt::message::Failure) -> Result<Self, Self::Error> {
        Ok(Failure {
            metadata: bolt_failure.metadata.try_into()?,
        })
    }
}

impl TryFrom<Message> for Failure {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Failure(failure) => Ok(Failure::try_from(failure)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
