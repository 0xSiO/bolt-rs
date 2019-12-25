use std::convert::TryFrom;

use failure::Error;

use crate::bolt;
use crate::bolt::Message;
use crate::error::MessageError;

#[derive(Debug)]
pub struct Reset;

impl TryFrom<bolt::message::Reset> for Reset {
    type Error = Error;

    fn try_from(_bolt_reset: bolt::message::Reset) -> Result<Self, Self::Error> {
        Ok(Reset)
    }
}

impl TryFrom<Message> for Reset {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Reset(reset) => Ok(Reset::try_from(reset)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
