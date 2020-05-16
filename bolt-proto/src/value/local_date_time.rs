use bolt_proto_derive::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x64;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct LocalDateTime {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use chrono::NaiveDate;

    use crate::serialization::*;
    use crate::value::integer::{MARKER_INT_16, MARKER_INT_64};

    use super::*;

    fn get_local_date_time() -> LocalDateTime {
        LocalDateTime::from(NaiveDate::from_ymd(2050, 3, 15).and_hms_nano(13, 15, 5, 420))
    }

    #[test]
    fn get_marker() {
        let local_date_time = get_local_date_time();
        assert_eq!(local_date_time.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let local_date_time = get_local_date_time();
        assert_eq!(
            local_date_time.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                MARKER_INT_64, // 64-bit integer since most significant bit is 1, don't want to interpret as negative
                0x00,
                0x00,
                0x00,
                0x00,
                0x96,
                0xDB,
                0x6D,
                0xD9,
                MARKER_INT_16,
                0x01,
                0xA4,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let local_date_time = get_local_date_time();
        let local_date_time_bytes = &[
            MARKER_INT_64,
            0x00,
            0x00,
            0x00,
            0x00,
            0x96,
            0xDB,
            0x6D,
            0xD9,
            MARKER_INT_16,
            0x01,
            0xA4,
        ];
        assert_eq!(
            LocalDateTime::try_from(Arc::new(Mutex::new(Bytes::from_static(
                local_date_time_bytes
            ))))
            .unwrap(),
            local_date_time
        );
    }
}
