use std::convert::TryInto;

use bytes::Bytes;

use crate::serialize::{Serialize, SerializeError, SerializeResult};

const MARKER: u8 = 0xC0;

#[derive(Debug)]
pub struct Null;

impl Serialize for Null {
    fn get_marker(&self) -> SerializeResult<u8> {
        Ok(MARKER)
    }
}

impl TryInto<Bytes> for Null {
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        self.get_marker().map(|m| Bytes::copy_from_slice(&[m]))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::serialize::Serialize;

    use super::{Null, MARKER};

    #[test]
    fn get_marker() {
        assert_eq!(Null.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        assert_eq!(
            Null.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER])
        );
    }
}
