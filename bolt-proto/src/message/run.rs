use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x10;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Run {
    pub(crate) statement: String,
    pub(crate) parameters: HashMap<String, Value>,
}

impl Run {
    pub fn new(statement: String, parameters: HashMap<String, Value>) -> Self {
        Self {
            statement,
            parameters,
        }
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }
}

impl_try_from_message!(Run, Run);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Run {
        Run::new(
            "RETURN $param;".to_string(),
            HashMap::from_iter(vec![("param".to_string(), Value::from(25_123_321_123_i64))]),
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
                MARKER_TINY_STRING | 14,
                b'R',
                b'E',
                b'T',
                b'U',
                b'R',
                b'N',
                b' ',
                b'$',
                b'p',
                b'a',
                b'r',
                b'a',
                b'm',
                b';',
                MARKER_TINY_MAP | 1,
                MARKER_TINY_STRING | 5,
                b'p',
                b'a',
                b'r',
                b'a',
                b'm',
                MARKER_INT_64,
                0x00,
                0x00,
                0x00,
                0x05,
                0xD9,
                0x77,
                0x75,
                0x23
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            MARKER_TINY_STRING | 14,
            b'R',
            b'E',
            b'T',
            b'U',
            b'R',
            b'N',
            b' ',
            b'$',
            b'p',
            b'a',
            b'r',
            b'a',
            b'm',
            b';',
            MARKER_TINY_MAP | 1,
            MARKER_TINY_STRING | 5,
            b'p',
            b'a',
            b'r',
            b'a',
            b'm',
            MARKER_INT_64,
            0x00,
            0x00,
            0x00,
            0x05,
            0xD9,
            0x77,
            0x75,
            0x23,
        ];
        assert_eq!(
            Run::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
