use chrono::{FixedOffset, NaiveTime, Offset, Timelike};

use bolt_proto_derive::*;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x54;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Time {
    pub(crate) nanos_since_midnight: i64,
    pub(crate) zone_offset: i32,
}

impl Time {
    pub fn naive_time(&self) -> NaiveTime {
        let seconds = (self.nanos_since_midnight / 1_000_000_000) as u32;
        let nanos = (self.nanos_since_midnight % 1_000_000_000) as u32;
        // Does not panic since seconds and nanos came from a NaiveTime already
        NaiveTime::from_num_seconds_from_midnight(seconds, nanos)
    }

    pub fn offset(&self) -> FixedOffset {
        FixedOffset::east(self.zone_offset)
    }
}

// No timezone-aware time in chrono, so provide separate conversion instead
impl<O: Offset> From<(NaiveTime, O)> for Time {
    fn from(pair: (NaiveTime, O)) -> Self {
        Self {
            // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
            nanos_since_midnight: pair.0.num_seconds_from_midnight() as i64 * 1_000_000_000
                + pair.0.nanosecond() as i64,
            zone_offset: pair.1.fix().local_minus_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::{MARKER_INT_16, MARKER_INT_64};

    use super::*;

    fn get_time() -> Time {
        Time::from((
            NaiveTime::from_hms_nano(1, 16, 40, 123),
            FixedOffset::east(3600),
        ))
    }

    #[test]
    fn get_marker() {
        assert_eq!(get_time().get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let time = get_time();
        assert_eq!(
            time.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                MARKER_INT_64,
                0x00,
                0x00,
                0x04,
                0x2F,
                0x05,
                0x5D,
                0xB0,
                0x7B,
                MARKER_INT_16,
                0x0E,
                0x10,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let time = get_time();
        let time_bytes = &[
            MARKER_INT_64,
            0x00,
            0x00,
            0x04,
            0x2F,
            0x05,
            0x5D,
            0xB0,
            0x7B,
            MARKER_INT_16,
            0x0E,
            0x10,
        ];
        assert_eq!(
            Time::try_from(Arc::new(Mutex::new(Bytes::from_static(time_bytes)))).unwrap(),
            time
        );
    }

    #[test]
    fn accessors() {
        let time = get_time();
        assert_eq!(time.naive_time(), NaiveTime::from_hms_nano(1, 16, 40, 123));
        assert_eq!(time.offset(), FixedOffset::east(3600));
    }
}
