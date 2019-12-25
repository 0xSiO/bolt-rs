use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::error::MessageError;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x0E;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct AckFailure;

impl From<native::message::AckFailure> for AckFailure {
    fn from(_native_ack_failure: native::message::AckFailure) -> Self {
        Self
    }
}

impl TryFrom<Message> for AckFailure {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::AckFailure(ack_failure) => Ok(ack_failure),
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
        let ack_failure = AckFailure::try_from(Arc::new(Mutex::new(bytes)));
        assert!(ack_failure.is_ok());
    }
}
