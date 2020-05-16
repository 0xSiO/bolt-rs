use bolt_proto_derive::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB4;
pub(crate) const SIGNATURE: u8 = 0x45;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Duration {
    pub(crate) months: i64,
    pub(crate) days: i64,
    pub(crate) seconds: i64,
    pub(crate) nanos: i32,
}

impl Duration {
    pub fn new(months: i64, days: i64, seconds: i64, nanos: i32) -> Self {
        Self {
            months,
            days,
            seconds,
            nanos,
        }
    }

    pub fn months(&self) -> i64 {
        self.months
    }

    pub fn days(&self) -> i64 {
        self.days
    }

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    pub fn nanos(&self) -> i32 {
        self.nanos
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::{MARKER_INT_16, MARKER_INT_32, MARKER_INT_64};

    use super::*;

    fn get_duration() -> Duration {
        Duration::new(7, 123_456_543, 54_213_945_693_251, 19287)
    }

    #[test]
    fn get_marker() {
        let duration = get_duration();
        assert_eq!(duration.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let duration = get_duration();
        assert_eq!(
            duration.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                0x07,
                MARKER_INT_32,
                0x07,
                0x5B,
                0xCC,
                0x1F,
                MARKER_INT_64,
                0x00,
                0x00,
                0x31,
                0x4E,
                0xAA,
                0xF9,
                0x94,
                0x43,
                MARKER_INT_16,
                0x4B,
                0x57,
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let duration = get_duration();
        let duration_bytes = &[
            0x07,
            MARKER_INT_32,
            0x07,
            0x5B,
            0xCC,
            0x1F,
            MARKER_INT_64,
            0x00,
            0x00,
            0x31,
            0x4E,
            0xAA,
            0xF9,
            0x94,
            0x43,
            MARKER_INT_16,
            0x4B,
            0x57,
        ];
        assert_eq!(
            Duration::try_from(Arc::new(Mutex::new(Bytes::from_static(duration_bytes)))).unwrap(),
            duration
        );
    }
}
