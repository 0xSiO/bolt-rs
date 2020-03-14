use std::panic::catch_unwind;

use chrono::{NaiveTime, Timelike};

use bolt_proto_derive::*;

use crate::error::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x54;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Time {
    pub(crate) nanos_since_midnight: i64,
    pub(crate) zone_offset: i32,
}

impl Time {
    pub fn new(
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
        zone_offset: (i32, i32),
    ) -> Result<Self> {
        let time = NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond)
            .ok_or(Error::InvalidTime(hour, minute, second, nanosecond))?;
        // Calculating the zone_offset may panic if it overflows
        let zone_offset = catch_unwind(|| zone_offset.0 * 3600 + zone_offset.1 * 60)
            .map_err(|_| Error::InvalidTimeZoneOffset(zone_offset))?;
        Ok(Self {
            nanos_since_midnight: time.num_seconds_from_midnight() as i64 * 1_000_000_000,
            zone_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use chrono::{DateTime, FixedOffset, NaiveDateTime};

    use crate::serialization::*;
    use crate::value::integer::{MARKER_INT_16, MARKER_INT_64};

    use super::*;

    fn get_chrono_date_time() -> DateTime<FixedOffset> {
        DateTime::from_utc(
            NaiveDateTime::from_timestamp(1000, 0),
            FixedOffset::east(3600),
        )
    }

    #[test]
    fn get_marker() {
        let time = Time::from(get_chrono_date_time());
        assert_eq!(time.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let time = Time::from(get_chrono_date_time());
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
                0x00,
                MARKER_INT_16,
                0x0E,
                0x10,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let time = Time::from(get_chrono_date_time());
        let time_bytes = &[
            MARKER_INT_64,
            0x00,
            0x00,
            0x04,
            0x2F,
            0x05,
            0x5D,
            0xB0,
            0x00,
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
    fn rejects_invalid_times() {
        assert!(Time::new(0, 0, 0, 0, (0, 0)).is_ok());
        assert!(Time::new(25, 0, 0, 0, (0, 0)).is_err());
        assert!(Time::new(0, 60, 0, 0, (0, 0)).is_err());
        assert!(Time::new(0, 0, 60, 0, (0, 0)).is_err());
        assert!(Time::new(0, 0, 0, 0, (i32::max_value(), 0)).is_err());
    }
}
