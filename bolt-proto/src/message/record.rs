use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x71;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Record {
    pub(crate) fields: Vec<Value>,
}

impl Record {
    pub fn new(fields: Vec<Value>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[Value] {
        &self.fields
    }
}

impl_try_from_message!(Record, Record);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Record {
        Record::new(vec![
            Value::from(1200_i16),
            Value::from("hi there"),
            Value::from(HashMap::<&str, Value>::from_iter(vec![(
                "key",
                Value::from("value"),
            )])),
        ])
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
                list::MARKER_TINY | 3,
                integer::MARKER_INT_16,
                0x04,
                0xB0,
                string::MARKER_TINY | 8,
                b'h',
                b'i',
                b' ',
                b't',
                b'h',
                b'e',
                b'r',
                b'e',
                map::MARKER_TINY | 1,
                string::MARKER_TINY | 3,
                b'k',
                b'e',
                b'y',
                string::MARKER_TINY | 5,
                b'v',
                b'a',
                b'l',
                b'u',
                b'e'
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            list::MARKER_TINY | 3,
            integer::MARKER_INT_16,
            0x04,
            0xB0,
            string::MARKER_TINY | 8,
            b'h',
            b'i',
            b' ',
            b't',
            b'h',
            b'e',
            b'r',
            b'e',
            map::MARKER_TINY | 1,
            string::MARKER_TINY | 3,
            b'k',
            b'e',
            b'y',
            string::MARKER_TINY | 5,
            b'v',
            b'a',
            b'l',
            b'u',
            b'e',
        ];
        assert_eq!(
            Record::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
