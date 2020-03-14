use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::{Message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x71;

#[derive(Debug, Clone, Signature, Marker, Serialize, Deserialize)]
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

impl TryFrom<Message> for Record {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Record(record) => Ok(record),
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
        let bytes = Bytes::from_static(&[0x93, 0x01, 0x02, 0x03]);
        assert!(Record::try_from(Arc::new(Mutex::new(bytes))).is_ok());
    }
}
