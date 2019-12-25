use std::convert::TryFrom;

use failure::Error;

use crate::bolt;
use crate::bolt::Message;
use crate::error::MessageError;

#[derive(Debug)]
pub struct AckFailure;

impl TryFrom<bolt::message::AckFailure> for AckFailure {
    type Error = Error;

    fn try_from(_bolt_ack_failure: bolt::message::AckFailure) -> Result<Self, Self::Error> {
        Ok(AckFailure)
    }
}

impl TryFrom<Message> for AckFailure {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::AckFailure(ack_failure) => Ok(AckFailure::try_from(ack_failure)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
