use std::convert::TryInto;

use bytes::Bytes;
use failure::Error;

use crate::serialize::{SerializeError, Value};

const MARKER: u8 = 0xC0;

#[derive(Debug)]
pub struct Null;

impl Value for Null {
    fn get_marker(&self) -> Result<u8, Error> {
        Ok(MARKER)
    }
}

impl TryInto<Bytes> for Null {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(Bytes::copy_from_slice(&[self.get_marker()?]))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::serialize::Value;

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
