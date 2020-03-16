use std::collections::HashMap;
use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::{Message, Value};

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x10;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
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

impl TryFrom<Message> for Run {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Run(run) => Ok(run),
            _ => Err(ConversionError::FromMessage(message).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use super::*;

    #[test]
    fn try_from_bytes() {
        let bytes = Bytes::from_static(&[
            0x8F, 0x52, 0x45, 0x54, 0x55, 0x52, 0x4E, 0x20, 0x31, 0x20, 0x41, 0x53, 0x20, 0x6E,
            0x75, 0x6D, 0xA0,
        ]);
        assert!(Run::try_from(Arc::new(Mutex::new(bytes))).is_ok());
    }
}
