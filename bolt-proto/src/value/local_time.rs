use chrono::{NaiveTime, Timelike};

use bolt_proto_derive::*;

use crate::error::*;
use crate::impl_try_from_value;

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x74;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct LocalTime {
    pub(crate) nanos_since_midnight: i64,
}

impl LocalTime {
    pub fn new(hour: u32, minute: u32, second: u32, nano: u32) -> Result<Self> {
        let time = NaiveTime::from_hms_nano_opt(hour, minute, second, nano)
            .ok_or(Error::InvalidTime(hour, minute, second, nano))?;
        Ok(Self {
            nanos_since_midnight: time.num_seconds_from_midnight() as i64 * 1_000_000_000,
        })
    }
}

impl From<NaiveTime> for LocalTime {
    fn from(naive_time: NaiveTime) -> Self {
        Self {
            nanos_since_midnight: naive_time.num_seconds_from_midnight() as i64 * 1_000_000_000,
        }
    }
}

impl_try_from_value!(LocalTime, LocalTime);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::MARKER_INT_64;

    use super::*;

    fn get_chrono_naive_time() -> NaiveTime {
        NaiveTime::from_hms(12, 34, 24)
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
            time.try_into_bytes().unwrap(),
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
                0x20,
                0x00,
            ])
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
            0x20,
            0x00,
        ];
        assert_eq!(
            LocalTime::try_from(Arc::new(Mutex::new(Bytes::from_static(time_bytes)))).unwrap(),
            time
        );
    }
}
