use std::convert::TryFrom;

use failure::Error;

use crate::bolt;
use crate::bolt::Message;
use crate::error::MessageError;

#[derive(Debug)]
pub struct DiscardAll;

impl TryFrom<bolt::message::DiscardAll> for DiscardAll {
    type Error = Error;

    fn try_from(_bolt_discard_all: bolt::message::DiscardAll) -> Result<Self, Self::Error> {
        Ok(DiscardAll)
    }
}

impl TryFrom<Message> for DiscardAll {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::DiscardAll(discard_all) => Ok(DiscardAll::try_from(discard_all)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
