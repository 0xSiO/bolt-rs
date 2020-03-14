use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use chrono_tz::Tz;

use bolt_proto_derive::*;

use crate::error::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x66;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeZoned {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) zone_id: String,
}

impl DateTimeZoned {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
        zone_id: String,
    ) -> Result<Self> {
        let date_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(Error::InvalidDate(year, month, day))?,
            NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond)
                .ok_or(Error::InvalidTime(hour, minute, second, nanosecond))?,
        );
        let timezone: Tz = zone_id
            .parse()
            .map_err(|_| Error::InvalidTimeZoneId(zone_id))?;
        Ok(Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            zone_id: timezone.name().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::{MARKER_INT_32, MARKER_INT_64};
    use crate::value::string;

    use super::*;

    fn get_date_time() -> DateTimeZoned {
        let zone = "Antarctica/Rothera".to_string();
        DateTimeZoned::new(3500, 7, 29, 13, 05, 01, 123456, zone).unwrap()
    }

    #[test]
    fn get_marker() {
        let time = get_date_time();
        assert_eq!(time.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let date_time_offset = get_date_time();
        assert_eq!(
            date_time_offset.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                MARKER_INT_64,
                0x00,
                0x00,
                0x00,
                0x0B,
                0x3E,
                0xEB,
                0x28,
                0xFD,
                MARKER_INT_32,
                0x00,
                0x01,
                0xE2,
                0x40,
                string::MARKER_SMALL,
                18,
                b'A',
                b'n',
                b't',
                b'a',
                b'r',
                b'c',
                b't',
                b'i',
                b'c',
                b'a',
                b'/',
                b'R',
                b'o',
                b't',
                b'h',
                b'e',
                b'r',
                b'a'
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let date_time_offset = get_date_time();
        let date_time_bytes = &[
            MARKER_INT_64,
            0x00,
            0x00,
            0x00,
            0x0B,
            0x3E,
            0xEB,
            0x28,
            0xFD,
            MARKER_INT_32,
            0x00,
            0x01,
            0xE2,
            0x40,
            string::MARKER_SMALL,
            18,
            b'A',
            b'n',
            b't',
            b'a',
            b'r',
            b'c',
            b't',
            b'i',
            b'c',
            b'a',
            b'/',
            b'R',
            b'o',
            b't',
            b'h',
            b'e',
            b'r',
            b'a',
        ];
        assert_eq!(
            DateTimeZoned::try_from(Arc::new(Mutex::new(Bytes::from_static(date_time_bytes))))
                .unwrap(),
            date_time_offset
        );
    }

    #[test]
    fn rejects_invalid_date_time() {
        assert!(DateTimeZoned::new(2015, 1, 1, 1, 1, 1, 1, "UTC".to_string()).is_ok());
        assert!(DateTimeZoned::new(2015, 13, 1, 1, 1, 1, 1, "UTC".to_string()).is_err());
        assert!(DateTimeZoned::new(2015, 1, 32, 1, 1, 1, 1, "UTC".to_string()).is_err());
        assert!(DateTimeZoned::new(2015, 1, 1, 1, 1, 1, 1, "INVALID".to_string()).is_err());
    }
}
