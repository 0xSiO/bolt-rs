use std::convert::TryInto;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, Bytes};
use failure::Error;
use failure::_core::convert::TryFrom;

use crate::bolt::value::{BoltValue, Marker};
use crate::error::DeserializeError;
use crate::serialize::{Deserialize, Serialize};

pub const MARKER: u8 = 0xC0;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Null;

impl From<Null> for BoltValue {
    fn from(value: Null) -> Self {
        BoltValue::Null(value)
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

impl TryFrom<Arc<Mutex<Bytes>>> for Null {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        let result: Result<Null, Error> = catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
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
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::bolt::value::Marker;
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

    #[test]
    fn try_from_bytes() {
        assert_eq!(
            Null::try_from(Arc::new(Mutex::new(Null.try_into_bytes().unwrap()))).unwrap(),
            Null
        );
        assert!(Null::try_from(Arc::new(Mutex::new(Bytes::from_static(&[0x01])))).is_err());
    }
}
