use chrono::{Duration, NaiveDate};

use bolt_proto_derive::*;

use crate::error::*;
use crate::impl_try_from_value;

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

impl From<NaiveDate> for Date {
    fn from(naive_date: NaiveDate) -> Self {
        // TODO: Pick between one of these methods
        println!(
            "Subtraction epoch days: {}",
            (naive_date - NaiveDate::from_ymd(1970, 1, 1)).num_days()
        );
        println!(
            "Using timestamp(): {}",
            naive_date.and_hms(0, 0, 0).timestamp() / Duration::days(1).num_seconds()
        );
        Self {
            // (seconds since epoch) / (seconds per day)
            days_since_epoch: (naive_date - NaiveDate::from_ymd(1970, 1, 1)).num_days(),
        }
    }
}

impl From<Date> for NaiveDate {
    fn from(date: Date) -> Self {
        NaiveDate::from_ymd(1970, 1, 1) + Duration::days(date.days_since_epoch)
    }
}

impl_try_from_value!(Date, Date);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::integer::MARKER_INT_16;

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
        let date = Date::new(1901, 12, 31).unwrap();
        let date_bytes = &[MARKER_INT_16, 0x9E, 0xFA];
        assert_eq!(
            Date::try_from(Arc::new(Mutex::new(Bytes::from_static(date_bytes)))).unwrap(),
            date
        );
    }
}
