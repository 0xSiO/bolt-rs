use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x01;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Init {
    pub(crate) client_name: String,
    pub(crate) auth_token: HashMap<String, Value>,
}

impl Init {
    pub fn new(client_name: String, auth_token: HashMap<String, Value>) -> Self {
        Self {
            client_name,
            auth_token,
        }
    }

    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    pub fn auth_token(&self) -> &HashMap<String, Value> {
        &self.auth_token
    }
}

impl_try_from_message!(Init, Init);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Init {
        Init::new(
            "MyClient/1.0".to_string(),
            HashMap::from_iter(vec![("scheme".to_string(), Value::from("basic"))]),
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
                MARKER_TINY_STRING | 12,
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
                MARKER_TINY_MAP | 1,
                MARKER_TINY_STRING | 6,
                b's',
                b'c',
                b'h',
                b'e',
                b'm',
                b'e',
                MARKER_TINY_STRING | 5,
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
            MARKER_TINY_STRING | 12,
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
            MARKER_TINY_MAP | 1,
            MARKER_TINY_STRING | 6,
            b's',
            b'c',
            b'h',
            b'e',
            b'm',
            b'e',
            MARKER_TINY_STRING | 5,
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
