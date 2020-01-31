use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::bolt::Value;
use crate::error::*;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x10;

#[derive(Debug, Clone, Signature, Marker, Serialize, Deserialize)]
pub struct Run {
    pub(crate) statement: Value,
    pub(crate) parameters: Value,
}

impl From<native::message::Run> for Run {
    fn from(native_run: native::message::Run) -> Self {
        Self {
            statement: Value::from(native_run.statement),
            parameters: Value::from(native_run.parameters),
        }
    }
}

impl TryFrom<Message> for Run {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Run(run) => Ok(run),
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
        let bytes = Bytes::from_static(&[
            0x8F, 0x52, 0x45, 0x54, 0x55, 0x52, 0x4E, 0x20, 0x31, 0x20, 0x41, 0x53, 0x20, 0x6E,
            0x75, 0x6D, 0xA0,
        ]);
        assert!(Run::try_from(Arc::new(Mutex::new(bytes))).is_ok());
    }
}
