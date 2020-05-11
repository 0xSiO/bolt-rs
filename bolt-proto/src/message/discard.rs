use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x2F;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Discard {
    pub(crate) metadata: HashMap<String, Value>,
}

impl Discard {
    pub fn new(metadata: HashMap<String, Value>) -> Self {
        Self { metadata }
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl_try_from_message!(Discard, Discard);

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Discard {
        Discard::new(HashMap::from_iter(vec![(
            "arbitrary".to_string(),
            Value::from("meh"),
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
                string::MARKER_TINY | 9,
                b'a',
                b'r',
                b'b',
                b'i',
                b't',
                b'r',
                b'a',
                b'r',
                b'y',
                string::MARKER_TINY | 3,
                b'm',
                b'e',
                b'h',
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            map::MARKER_TINY | 1,
            string::MARKER_TINY | 9,
            b'a',
            b'r',
            b'b',
            b'i',
            b't',
            b'r',
            b'a',
            b'r',
            b'y',
            string::MARKER_TINY | 3,
            b'm',
            b'e',
            b'h',
        ];
        assert_eq!(
            Discard::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
