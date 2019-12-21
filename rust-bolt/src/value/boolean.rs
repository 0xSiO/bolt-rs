use std::convert::TryInto;

use bytes::Bytes;
use failure::Error;

use crate::serialize::Value;

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

impl Value for Boolean {
    fn get_marker(&self) -> Result<u8, Error> {
        if self.value {
            Ok(MARKER_TRUE)
        } else {
            Ok(MARKER_FALSE)
        }
    }
}

impl TryInto<Bytes> for Boolean {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(Bytes::copy_from_slice(&[self.get_marker()?]))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::serialize::Value;

    use super::{Boolean, MARKER_FALSE, MARKER_TRUE};

    #[test]
    fn get_marker() {
        let f = Boolean::from(false);
        assert_eq!(f.get_marker().unwrap(), MARKER_FALSE);
        let t = Boolean::from(true);
        assert_eq!(t.get_marker().unwrap(), MARKER_TRUE);
    }

    #[test]
    fn try_into_bytes() {
        let f = Boolean::from(false);
        assert_eq!(
            f.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_FALSE])
        );
        let t = Boolean::from(true);
        assert_eq!(
            t.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_TRUE])
        );
    }
}
