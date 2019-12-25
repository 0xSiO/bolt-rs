use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::error::MessageError;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x2F;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct DiscardAll;

impl From<native::message::DiscardAll> for DiscardAll {
    fn from(_native_discard_all: native::message::DiscardAll) -> Self {
        Self
    }
}

impl TryFrom<Message> for DiscardAll {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::DiscardAll(discard_all) => Ok(discard_all),
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
        let discard_all = DiscardAll::try_from(Arc::new(Mutex::new(bytes)));
        assert!(discard_all.is_ok());
    }
}
