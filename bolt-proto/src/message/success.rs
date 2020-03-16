use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x70;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Success {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Success {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl_try_from_message!(Success, Success);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Success {
        Success::new(HashMap::from_iter(vec![(
            "some key".to_string(),
            Value::from(vec![1_i8, -2_i8, 3_i8]),
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
                map::MARKER_TINY | 1,
                string::MARKER_TINY | 8,
                b's',
                b'o',
                b'm',
                b'e',
                b' ',
                b'k',
                b'e',
                b'y',
                list::MARKER_TINY | 3,
                0x01,
                0xFE,
                0x03,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            map::MARKER_TINY | 1,
            string::MARKER_TINY | 8,
            b's',
            b'o',
            b'm',
            b'e',
            b' ',
            b'k',
            b'e',
            b'y',
            list::MARKER_TINY | 3,
            0x01,
            0xFE,
            0x03,
        ];
        assert_eq!(
            Success::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
