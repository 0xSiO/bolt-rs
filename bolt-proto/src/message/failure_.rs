use std::collections::HashMap;
use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::{Message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x7F;

#[derive(Debug, Clone, Signature, Marker, Serialize, Deserialize)]
pub struct Failure {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Failure {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl TryFrom<Message> for Failure {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Failure(failure) => Ok(failure),
            _ => Err(Error::InvalidMessageConversion(message).into()),
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
            0xA2, 0x84, 0x63, 0x6F, 0x64, 0x65, 0xD0, 0x25, 0x4E, 0x65, 0x6F, 0x2E, 0x43, 0x6C,
            0x69, 0x65, 0x6E, 0x74, 0x45, 0x72, 0x72, 0x6F, 0x72, 0x2E, 0x53, 0x74, 0x61, 0x74,
            0x65, 0x6D, 0x65, 0x6E, 0x74, 0x2E, 0x53, 0x79, 0x6E, 0x74, 0x61, 0x78, 0x45, 0x72,
            0x72, 0x6F, 0x72, 0x87, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x8F, 0x49, 0x6E,
            0x76, 0x61, 0x6C, 0x69, 0x64, 0x20, 0x73, 0x79, 0x6E, 0x74, 0x61, 0x78, 0x2E,
        ]);
        let failure = Failure::try_from(Arc::new(Mutex::new(bytes)));
        assert!(failure.is_ok());
    }
}
