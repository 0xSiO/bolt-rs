use chrono::NaiveTime;

use bolt_proto_derive::*;

use crate::error::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x74;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct LocalTime {
    pub(crate) nanos_since_midnight: i64,
}

impl LocalTime {
    pub fn new(hour: u32, minute: u32, second: u32, nano: u32) -> Result<Self> {
        let naive_time = NaiveTime::from_hms_nano_opt(hour, minute, second, nano)
            .ok_or(Error::InvalidTime(hour, minute, second, nano))?;
        Ok(LocalTime::from(naive_time))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::MARKER_INT_64;

    use super::*;

    fn get_chrono_naive_time() -> NaiveTime {
        NaiveTime::from_hms_nano(12, 34, 24, 1029)
    }

    #[test]
    fn get_marker() {
        let time = LocalTime::from(get_chrono_naive_time());
        assert_eq!(time.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let time = LocalTime::from(get_chrono_naive_time());
        assert_eq!(
            time.try_into_bytes().unwrap().to_vec(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                MARKER_INT_64,
                0x00,
                0x00,
                0x29,
                0x2A,
                0xD8,
                0xA4,
                0x24,
                0x05,
            ])
            .to_vec()
        );
    }

    #[test]
    fn try_from_bytes() {
        let time = LocalTime::from(get_chrono_naive_time());
        let time_bytes = &[
            MARKER_INT_64,
            0x00,
            0x00,
            0x29,
            0x2A,
            0xD8,
            0xA4,
            0x24,
            0x05,
        ];
        assert_eq!(
            LocalTime::try_from(Arc::new(Mutex::new(Bytes::from_static(time_bytes)))).unwrap(),
            time
        );
    }

    #[test]
    fn rejects_invalid_times() {
        assert!(LocalTime::new(0, 0, 0, 0).is_ok());
        assert!(LocalTime::new(25, 0, 0, 0).is_err());
        assert!(LocalTime::new(0, 60, 0, 0).is_err());
        assert!(LocalTime::new(0, 0, 60, 0).is_err());
        assert!(LocalTime::new(u32::MAX, 0, 0, 0).is_err());
    }
}
