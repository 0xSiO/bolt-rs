use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::error::MessageError;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x7E;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct Ignored;

impl From<native::message::Ignored> for Ignored {
    fn from(_native_ignored: native::message::Ignored) -> Self {
        Self
    }
}

impl TryFrom<Message> for Ignored {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Ignored(ignored) => Ok(ignored),
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
        let ignored = Ignored::try_from(Arc::new(Mutex::new(bytes)));
        assert!(ignored.is_ok());
    }
}
