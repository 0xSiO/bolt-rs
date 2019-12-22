use std::convert::{TryFrom, TryInto};
use std::panic::catch_unwind;

use bytes::{Buf, Bytes};
use failure::Error;

use crate::error::DeserializeError;
use crate::serialize::{Deserialize, Serialize};
use crate::value::{Marker, Value};
use std::sync::Mutex;

pub const MARKER_FALSE: u8 = 0xC2;
pub const MARKER_TRUE: u8 = 0xC3;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Boolean {
    value: bool,
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value.into())
    }
}

impl Marker for Boolean {
    fn get_marker(&self) -> Result<u8, Error> {
        if self.value {
            Ok(MARKER_TRUE)
        } else {
            Ok(MARKER_FALSE)
        }
    }
}

impl Serialize for Boolean {}

impl TryInto<Bytes> for Boolean {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(Bytes::copy_from_slice(&[self.get_marker()?]))
    }
}

impl Deserialize<'_> for Boolean {}

impl TryFrom<&mut Bytes> for Boolean {
    type Error = Error;

    fn try_from(input_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let input_bytes = Mutex::new(input_bytes);
        let result: Result<Boolean, Error> = catch_unwind(move || {
            let mut input_bytes = input_bytes.lock().unwrap();
            let marker = input_bytes.get_u8();
            debug_assert!(!input_bytes.has_remaining());
            match marker {
                MARKER_TRUE => Ok(Boolean::from(true)),
                MARKER_FALSE => Ok(Boolean::from(false)),
                _ => Err(DeserializeError(format!("Invalid marker byte: {:x}", marker)).into()),
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Boolean from Bytes: {}", err))
        })?)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use bytes::Bytes;

    use crate::serialize::Serialize;
    use crate::value::Marker;

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

    #[test]
    fn try_from_bytes() {
        let f = Boolean::from(false);
        assert_eq!(
            Boolean::try_from(&mut f.clone().try_into_bytes().unwrap()).unwrap(),
            f
        );
        let t = Boolean::from(true);
        assert_eq!(
            Boolean::try_from(&mut t.clone().try_into_bytes().unwrap()).unwrap(),
            t
        );
        assert!(Boolean::try_from(&mut Bytes::from_static(&[0x01])).is_err());
    }
}
