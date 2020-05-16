use bolt_proto_derive::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x46;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeOffset {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) offset_seconds: i32,
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use chrono::{DateTime, FixedOffset, NaiveDateTime};

    use crate::serialization::*;
    use crate::value::integer::MARKER_INT_16;

    use super::*;

    fn get_chrono_date_time() -> DateTime<FixedOffset> {
        DateTime::from_utc(
            NaiveDateTime::from_timestamp(2000, 1000),
            FixedOffset::east(-1200),
        )
    }

    #[test]
    fn get_marker() {
        let time = DateTimeOffset::from(get_chrono_date_time());
        assert_eq!(time.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let date_time_offset = DateTimeOffset::from(get_chrono_date_time());
        assert_eq!(
            date_time_offset.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                MARKER_INT_16,
                0x07,
                0xD0,
                MARKER_INT_16,
                0x03,
                0xE8,
                MARKER_INT_16,
                0xFB,
                0x50,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let date_time_offset = DateTimeOffset::from(get_chrono_date_time());
        let date_time_bytes = &[
            MARKER_INT_16,
            0x07,
            0xD0,
            MARKER_INT_16,
            0x03,
            0xE8,
            MARKER_INT_16,
            0xFB,
            0x50,
        ];
        assert_eq!(
            DateTimeOffset::try_from(Arc::new(Mutex::new(Bytes::from_static(date_time_bytes))))
                .unwrap(),
            date_time_offset
        );
    }
}
