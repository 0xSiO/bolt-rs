use std::convert::TryFrom;

use failure::Error;

use crate::bolt;
use crate::bolt::Message;
use crate::error::MessageError;

#[derive(Debug)]
pub struct Ignored;

impl TryFrom<bolt::message::Ignored> for Ignored {
    type Error = Error;

    fn try_from(_bolt_ignored: bolt::message::Ignored) -> Result<Self, Self::Error> {
        Ok(Ignored)
    }
}

impl TryFrom<Message> for Ignored {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Ignored(ignored) => Ok(Ignored::try_from(ignored)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
