use bolt_proto_derive::*;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::error::*;
use crate::Message;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x01;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
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
            _ => Err(ConversionError::FromMessage(message).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Init {
        Init::new(
            "MyClient/1.0".to_string(),
            HashMap::from_iter(vec![("scheme".to_string(), "basic".to_string())]),
        )
    }

    #[test]
    fn get_marker() {
        assert_eq!(new_msg().get_marker().unwrap(), MARKER);
    }

    #[test]
    fn get_signature() {
        assert_eq!(new_msg().get_signature(), SIGNATURE);
    }

    #[test]
    fn try_into_bytes() {
        let msg = new_msg();
        assert_eq!(
            msg.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                string::MARKER_TINY | 12,
                b'M',
                b'y',
                b'C',
                b'l',
                b'i',
                b'e',
                b'n',
                b't',
                b'/',
                b'1',
                b'.',
                b'0',
                map::MARKER_TINY | 1,
                string::MARKER_TINY | 6,
                b's',
                b'c',
                b'h',
                b'e',
                b'm',
                b'e',
                string::MARKER_TINY | 5,
                b'b',
                b'a',
                b's',
                b'i',
                b'c'
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            string::MARKER_TINY | 12,
            b'M',
            b'y',
            b'C',
            b'l',
            b'i',
            b'e',
            b'n',
            b't',
            b'/',
            b'1',
            b'.',
            b'0',
            map::MARKER_TINY | 1,
            string::MARKER_TINY | 6,
            b's',
            b'c',
            b'h',
            b'e',
            b'm',
            b'e',
            string::MARKER_TINY | 5,
            b'b',
            b'a',
            b's',
            b'i',
            b'c',
        ];
        assert_eq!(
            Init::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
