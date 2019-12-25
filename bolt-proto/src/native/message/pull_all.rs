use std::convert::TryFrom;

use failure::Error;

use crate::bolt;
use crate::bolt::Message;
use crate::error::MessageError;

#[derive(Debug)]
pub struct PullAll;

impl TryFrom<bolt::message::PullAll> for PullAll {
    type Error = Error;

    fn try_from(_bolt_pull_all: bolt::message::PullAll) -> Result<Self, Self::Error> {
        Ok(PullAll)
    }
}

impl TryFrom<Message> for PullAll {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::PullAll(pull_all) => Ok(PullAll::try_from(pull_all)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
