use std::convert::TryInto;

use bytes::Bytes;

use crate::serialize::{Serialize, SerializeError, SerializeResult};

const MARKER_FALSE: u8 = 0xC2;
const MARKER_TRUE: u8 = 0xC3;

#[derive(Debug)]
pub struct Boolean {
    value: bool,
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl Serialize for Boolean {
    fn get_marker(&self) -> SerializeResult<u8> {
        if self.value {
            Ok(MARKER_TRUE)
        } else {
            Ok(MARKER_FALSE)
        }
    }
}

impl TryInto<Bytes> for Boolean {
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        self.get_marker().map(|m| Bytes::copy_from_slice(&[m]))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::serialize::Serialize;

    use super::{Boolean, MARKER_FALSE, MARKER_TRUE};

    #[test]
    fn is_valid() {
        let f = Boolean::from(false);
        assert_eq!(f.get_marker().unwrap(), MARKER_FALSE);
        assert_eq!(
            f.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_FALSE])
        );
        let t = Boolean::from(true);
        assert_eq!(t.get_marker().unwrap(), MARKER_TRUE);
        assert_eq!(
            t.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_TRUE])
        );
    }
}
