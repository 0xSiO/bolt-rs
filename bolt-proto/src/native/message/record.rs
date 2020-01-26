use std::convert::{TryFrom, TryInto};

use crate::bolt;
use crate::bolt::Message;
use crate::bolt::Value;
use crate::error::*;

#[derive(Debug)]
pub struct Record {
    pub(crate) fields: Vec<Value>,
}

impl Record {
    pub fn new(fields: Vec<Value>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[Value] {
        &self.fields
    }
}

impl TryFrom<bolt::message::Record> for Record {
    type Error = Error;

    fn try_from(bolt_record: bolt::message::Record) -> Result<Self> {
        Ok(Record {
            fields: bolt_record.fields.try_into()?,
        })
    }
}

impl TryFrom<Message> for Record {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Record(record) => Ok(Record::try_from(record)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
