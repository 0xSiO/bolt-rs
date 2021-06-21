use chrono::{NaiveDateTime, Timelike};
use chrono_tz::Tz;

use bolt_proto_derive::*;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x66;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeZoned {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) zone_id: String,
}

// Can't impl<T: TimeZone> From<DateTime<T>> for DateTimeZoned, since we can't get a
// timezone name from an Offset. Provide separate conversion instead
impl From<(NaiveDateTime, Tz)> for DateTimeZoned {
    fn from(pair: (NaiveDateTime, Tz)) -> Self {
        Self {
            epoch_seconds: pair.0.timestamp(),
            nanos: pair.0.nanosecond() as i64,
            zone_id: pair.1.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use chrono::NaiveDate;

    use crate::serialization::*;
    use crate::value::{string, MARKER_INT_32, MARKER_INT_64};

    use super::*;

    fn get_date_time() -> DateTimeZoned {
        DateTimeZoned::from((
            NaiveDate::from_ymd(3500, 7, 29).and_hms_nano(13, 5, 1, 123_456),
            chrono_tz::Antarctica::Rothera,
        ))
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
}
