use std::collections::HashMap;
use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::Message;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x01;

#[derive(Debug, Clone, Signature, Marker, Serialize, Deserialize)]
pub struct Init {
    pub(crate) client_name: String,
    pub(crate) auth_token: HashMap<String, String>,
}

impl Init {
    pub fn new(client_name: String, auth_token: HashMap<String, String>) -> Self {
        Self {
            client_name,
            auth_token,
        }
    }

    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    pub fn auth_token(&self) -> &HashMap<String, String> {
        &self.auth_token
    }
}

impl TryFrom<Message> for Init {
    type Error = Error;

    fn try_from(message: Message) -> Result<Self> {
        match message {
            Message::Init(init) => Ok(init),
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

    fn new_msg() -> Init {
        Init {
            client_name: String::from("MyClient/1.0"),
            auth_token: HashMap::from_iter(vec![("scheme".to_string(), "basic".to_string())]),
        }
    }

    #[test]
    fn get_marker() {
        assert_eq!(new_msg().get_marker().unwrap(), 0xB2);
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
                0xB2, 0x01, 0x8C, 0x4D, 0x79, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x2F, 0x31, 0x2E,
                0x30, 0xA1, 0x86, 0x73, 0x63, 0x68, 0x65, 0x6D, 0x65, 0x85, 0x62, 0x61, 0x73, 0x69,
                0x63,
            ])
        );
    }
}
