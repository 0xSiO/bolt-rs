use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x01;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Hello {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Hello);
impl_try_from_message!(Hello, Hello);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Hello {
        Hello::new(HashMap::from_iter(vec![(
            "user_agent".to_string(),
            Value::from("MyClient/1.0"),
        )]))
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
                MARKER_TINY_MAP | 1,
                MARKER_TINY_STRING | 10,
                b'u',
                b's',
                b'e',
                b'r',
                b'_',
                b'a',
                b'g',
                b'e',
                b'n',
                b't',
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
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            MARKER_TINY_MAP | 1,
            MARKER_TINY_STRING | 10,
            b'u',
            b's',
            b'e',
            b'r',
            b'_',
            b'a',
            b'g',
            b'e',
            b'n',
            b't',
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
        ];
        assert_eq!(
            Hello::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
