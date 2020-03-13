use std::convert::{TryFrom, TryInto};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, Bytes};

use crate::error::*;
use crate::serialization::*;

mod conversions;

pub(crate) const MARKER_FALSE: u8 = 0xC2;
pub(crate) const MARKER_TRUE: u8 = 0xC3;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) struct Boolean {
    pub(crate) value: bool,
}

impl Marker for Boolean {
    fn get_marker(&self) -> Result<u8> {
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

    fn try_into(self) -> Result<Bytes> {
        Ok(Bytes::copy_from_slice(&[self.get_marker()?]))
    }
}

impl Deserialize for Boolean {}

impl TryFrom<Arc<Mutex<Bytes>>> for Boolean {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self> {
        catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
            let marker = input_bytes.get_u8();
            debug_assert!(!input_bytes.has_remaining());
            match marker {
                MARKER_TRUE => Ok(Boolean::from(true)),
                MARKER_FALSE => Ok(Boolean::from(false)),
                _ => Err(DeserializationError::InvalidMarkerByte(marker).into()),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use super::*;

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
            Boolean::try_from(Arc::new(Mutex::new(f.clone().try_into_bytes().unwrap()))).unwrap(),
            f
        );
        let t = Boolean::from(true);
        assert_eq!(
            Boolean::try_from(Arc::new(Mutex::new(t.clone().try_into_bytes().unwrap()))).unwrap(),
            t
        );
        assert!(Boolean::try_from(Arc::new(Mutex::new(Bytes::from_static(&[0x01])))).is_err());
    }
}
