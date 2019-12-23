use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};
use std::mem;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use failure::Error;

use crate::bolt::value::{BoltValue, Marker};
use crate::error::{DeserializeError, ValueError};
use crate::serialize::{Deserialize, Serialize};

pub const MARKER: u8 = 0xC1;

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
}

impl Hash for Float {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!("Cannot hash a Float")
    }
}

impl Eq for Float {
    fn assert_receiver_is_total_eq(&self) {
        panic!("Floats cannot be Eq")
    }
}

impl From<f64> for Float {
    fn from(float: f64) -> Self {
        Self { value: float }
    }
}

impl TryFrom<BoltValue> for Float {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::Float(float) => Ok(float),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

impl From<f64> for BoltValue {
    fn from(value: f64) -> Self {
        BoltValue::Float(value.into())
    }
}

impl Marker for Float {
    fn get_marker(&self) -> Result<u8, Error> {
        Ok(MARKER)
    }
}

impl Serialize for Float {}

impl TryInto<Bytes> for Float {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        let mut bytes = BytesMut::with_capacity(mem::size_of::<u8>() * 9);
        bytes.put_u8(MARKER);
        bytes.put_f64(self.value);
        Ok(bytes.freeze())
    }
}

impl Deserialize for Float {}

impl TryFrom<Arc<Mutex<Bytes>>> for Float {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        let result: Result<Float, Error> = catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
            let marker = input_bytes.get_u8();

            match marker {
                MARKER => Ok(Float::from(input_bytes.get_f64())),
                _ => Err(DeserializeError(format!("Invalid marker byte: {:x}", marker)).into()),
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Integer from Bytes: {}", err))
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

    use super::*;

    #[test]
    fn get_marker() {
        let min = Float::from(std::f64::MIN_POSITIVE);
        assert_eq!(min.get_marker().unwrap(), MARKER);
        let e = Float::from(std::f64::consts::E);
        assert_eq!(e.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let pi = Float::from(std::f64::consts::PI);
        assert_eq!(
            pi.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER, 64, 9, 33, 251, 84, 68, 45, 24])
        );
    }

    #[test]
    fn try_from_bytes() {
        let pi = Float::from(std::f64::consts::PI);
        assert_eq!(
            Float::try_from(Arc::new(Mutex::new(pi.clone().try_into_bytes().unwrap()))).unwrap(),
            pi
        );
        let max = Float::from(std::f64::MAX);
        assert_eq!(
            Float::try_from(Arc::new(Mutex::new(max.clone().try_into_bytes().unwrap()))).unwrap(),
            max
        );
        assert!(Float::try_from(Arc::new(Mutex::new(Bytes::from_static(&[0x01])))).is_err());
    }
}
