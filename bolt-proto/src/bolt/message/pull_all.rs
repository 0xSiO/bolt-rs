use std::convert::TryFrom;

use failure::Error;

use bolt_proto_derive::*;

use crate::bolt::Message;
use crate::error::MessageError;
use crate::native;

pub(crate) const SIGNATURE: u8 = 0x3F;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct PullAll;

impl From<native::message::PullAll> for PullAll {
    fn from(_native_pull_all: native::message::PullAll) -> Self {
        Self
    }
}

impl TryFrom<Message> for PullAll {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::PullAll(pull_all) => Ok(pull_all),
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
        let pull_all = PullAll::try_from(Arc::new(Mutex::new(bytes)));
        assert!(pull_all.is_ok());
    }
}
