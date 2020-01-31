use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::bolt::Value;
use crate::error::*;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x71;

#[derive(Debug, Clone, Signature, Marker, Serialize, Deserialize)]
pub struct Record {
    pub(crate) fields: Value,
}

impl From<native::message::Record> for Record {
    fn from(native_record: native::message::Record) -> Self {
        Self {
            fields: Value::from(native_record.fields),
        }
    }
}

impl TryFrom<Message> for Record {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Record(record) => Ok(record),
            _ => Err(MessageError::InvalidConversion(message).into()),
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
