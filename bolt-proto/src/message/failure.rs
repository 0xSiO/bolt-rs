use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_message_with_metadata, impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x7F;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Failure {
    pub(crate) metadata: HashMap<String, Value>,
}

impl_message_with_metadata!(Failure);
impl_try_from_message!(Failure, Failure);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use chrono::NaiveDate;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> Failure {
        Failure::new(HashMap::from_iter(vec![(
            "failing_since".to_string(),
            Value::from(NaiveDate::from_ymd(1985, 6, 26)),
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
                string::MARKER_TINY | 13,
                b'f',
                b'a',
                b'i',
                b'l',
                b'i',
                b'n',
                b'g',
                b'_',
                b's',
                b'i',
                b'n',
                b'c',
                b'e',
                date::MARKER,
                date::SIGNATURE,
                MARKER_INT_16,
                0x16,
                0x17,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            map::MARKER_TINY | 1,
            string::MARKER_TINY | 13,
            b'f',
            b'a',
            b'i',
            b'l',
            b'i',
            b'n',
            b'g',
            b'_',
            b's',
            b'i',
            b'n',
            b'c',
            b'e',
            date::MARKER,
            date::SIGNATURE,
            MARKER_INT_16,
            0x16,
            0x17,
        ];
        assert_eq!(
            Failure::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
