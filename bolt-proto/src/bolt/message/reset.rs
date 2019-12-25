use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::error::MessageError;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x0F;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct Reset;

impl From<native::message::Reset> for Reset {
    fn from(_native_reset: native::message::Reset) -> Self {
        Self
    }
}

impl TryFrom<Message> for Reset {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Reset(reset) => Ok(reset),
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
        // No data needed!
        let bytes = Bytes::from_static(&[]);
        let reset = Reset::try_from(Arc::new(Mutex::new(bytes)));
        assert!(reset.is_ok());
    }
}
