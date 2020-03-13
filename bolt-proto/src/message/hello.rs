use std::collections::HashMap;
use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::{Message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x01;

#[derive(Debug, Clone, Signature, Marker, Serialize, Deserialize)]
pub struct Hello {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Hello {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl TryFrom<Message> for Hello {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Hello(hello) => Ok(hello),
            _ => Err(Error::InvalidMessageConversion(message).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use bytes::Bytes;

    use crate::serialization::*;

    use super::*;

    fn new_msg() -> Hello {
        Hello {
            metadata: HashMap::from_iter(vec![(
                "arbitrary".to_string(),
                Value::from("any kind of metadata"),
            )]),
        }
    }

    #[test]
    fn get_marker() {
        assert_eq!(new_msg().get_marker().unwrap(), 0xB1);
    }

    #[test]
    fn get_signature() {
        assert_eq!(new_msg().get_signature(), 0x01);
    }

    #[test]
    fn try_into_bytes() {
        assert_eq!(
            new_msg().try_into_bytes().unwrap(),
            Bytes::from_static(&[
                0xB1, 0x01, 0xA1, 0x89, 0x61, 0x72, 0x62, 0x69, 0x74, 0x72, 0x61, 0x72, 0x79, 0xD0,
                0x14, 0x61, 0x6E, 0x79, 0x20, 0x6B, 0x69, 0x6E, 0x64, 0x20, 0x6F, 0x66, 0x20, 0x6D,
                0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61
            ])
        );
    }
}
