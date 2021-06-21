use bolt_proto_derive::*;
use chrono::{NaiveTime, Timelike};

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x74;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct LocalTime {
    pub(crate) nanos_since_midnight: i64,
}

impl From<NaiveTime> for LocalTime {
    fn from(naive_time: NaiveTime) -> Self {
        Self {
            // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
            nanos_since_midnight: naive_time.num_seconds_from_midnight() as i64 * 1_000_000_000
                + naive_time.nanosecond() as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use chrono::NaiveTime;

    use crate::serialization::*;
    use crate::value::MARKER_INT_64;

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
}
