use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bolt;
use crate::bolt::Message;
use crate::bolt::Value;
use crate::error::Error;
use crate::error::MessageError;

#[derive(Debug)]
pub struct Run {
    pub(crate) statement: String,
    pub(crate) parameters: HashMap<String, Value>,
}

impl Run {
    pub fn new(statement: String, parameters: HashMap<String, Value>) -> Self {
        Self {
            statement,
            parameters,
        }
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }
}

impl TryFrom<bolt::message::Run> for Run {
    type Error = Error;

    fn try_from(bolt_run: bolt::message::Run) -> Result<Self, Self::Error> {
        Ok(Run {
            statement: bolt_run.statement.try_into()?,
            parameters: bolt_run.parameters.try_into()?,
        })
    }
}

impl TryFrom<Message> for Run {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Run(run) => Ok(Run::try_from(run)?),
            _ => Err(MessageError::InvalidConversion(message).into()),
        }
    }
}
