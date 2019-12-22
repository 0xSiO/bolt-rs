use std::convert::TryInto;
use std::panic::catch_unwind;

use bytes::{Buf, Bytes};
use failure::Error;
use failure::_core::convert::TryFrom;

use crate::error::DeserializeError;
use crate::serialize::{Deserialize, Serialize};
use crate::value::{Marker, Value};

const MARKER: u8 = 0xC0;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Null;

impl From<Null> for Value {
    fn from(value: Null) -> Self {
        Value::Null(value)
    }
}

impl Marker for Null {
    fn get_marker(&self) -> Result<u8, Error> {
        Ok(MARKER)
    }
}

impl Serialize for Null {}

impl TryInto<Bytes> for Null {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(Bytes::copy_from_slice(&[self.get_marker()?]))
    }
}

impl Deserialize for Null {}

impl TryFrom<Bytes> for Null {
    type Error = Error;

    fn try_from(mut input_bytes: Bytes) -> Result<Self, Self::Error> {
        let result: Result<Null, Error> = catch_unwind(move || {
            let marker = input_bytes.get_u8();
            debug_assert!(!input_bytes.has_remaining());
            if marker == MARKER {
                Ok(Null)
            } else {
                Err(DeserializeError(format!("Invalid marker byte: {:x}", marker)).into())
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Null from Bytes: {}", err))
        })?)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::serialize::Serialize;
    use crate::value::Marker;

    use super::{Null, MARKER};
    use std::convert::TryFrom;

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

    #[test]
    fn try_from_bytes() {
        assert_eq!(
            Null::try_from(Null.try_into_bytes().unwrap()).unwrap(),
            Null
        );
        assert!(Null::try_from(Bytes::from_static(&[0x01])).is_err());
    }
}
