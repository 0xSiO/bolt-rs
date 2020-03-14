use chrono::NaiveDate;

use bolt_proto_derive::*;

use crate::error::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x44;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Date {
    pub(crate) days_since_epoch: i64,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self> {
        Ok(Self::from(
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(Error::InvalidDate(year, month, day))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::{MARKER_INT_16, MARKER_INT_32};

    use super::*;

    #[test]
    fn get_marker() {
        let date = Date::from(NaiveDate::from_ymd(2020, 01, 01));
        assert_eq!(date.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let date = Date::new(1901, 12, 31).unwrap();
        assert_eq!(
            date.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER, SIGNATURE, MARKER_INT_16, 0x9E, 0xFA])
        );
    }

    #[test]
    fn try_from_bytes() {
        let past_date = Date::new(1901, 12, 31).unwrap();
        let past_bytes = &[MARKER_INT_16, 0x9E, 0xFA];
        let future_date = Date::new(3000, 5, 23).unwrap();
        let future_bytes = &[MARKER_INT_32, 0x00, 0x05, 0xBE, 0x16];
        assert_eq!(
            Date::try_from(Arc::new(Mutex::new(Bytes::from_static(past_bytes)))).unwrap(),
            past_date
        );
        assert_eq!(
            Date::try_from(Arc::new(Mutex::new(Bytes::from_static(future_bytes)))).unwrap(),
            future_date
        );
    }

    #[test]
    fn rejects_invalid_date() {
        assert!(Date::new(2019, 1, 0).is_err());
        assert!(Date::new(2000, 13, 1).is_err());
    }
}
