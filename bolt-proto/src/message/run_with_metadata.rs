use std::collections::HashMap;

use bolt_proto_derive::*;

use crate::{impl_try_from_message, Value};

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x10;

#[derive(Debug, Clone, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct RunWithMetadata {
    pub(crate) statement: String,
    pub(crate) parameters: HashMap<String, Value>,
    pub(crate) metadata: HashMap<String, Value>,
}

impl RunWithMetadata {
    pub fn new(
        statement: String,
        parameters: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
    ) -> Self {
        Self {
            statement,
            parameters,
            metadata,
        }
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }

    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
}

impl_try_from_message!(RunWithMetadata, RunWithMetadata);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::*;

    use super::*;

    fn new_msg() -> RunWithMetadata {
        RunWithMetadata::new(
            "something;".to_string(),
            HashMap::new(),
            HashMap::from_iter(vec![("arbitrary".to_string(), Value::from("any"))]),
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
                string::MARKER_TINY | 10,
                b's',
                b'o',
                b'm',
                b'e',
                b't',
                b'h',
                b'i',
                b'n',
                b'g',
                b';',
                MARKER_TINY_MAP,
                MARKER_TINY_MAP | 1,
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
                b'a',
                b'n',
                b'y',
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = new_msg();
        let msg_bytes = &[
            string::MARKER_TINY | 10,
            b's',
            b'o',
            b'm',
            b'e',
            b't',
            b'h',
            b'i',
            b'n',
            b'g',
            b';',
            MARKER_TINY_MAP,
            MARKER_TINY_MAP | 1,
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
            b'a',
            b'n',
            b'y',
        ];
        assert_eq!(
            RunWithMetadata::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
