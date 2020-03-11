use std::convert::TryFrom;

use chrono::{Duration, NaiveDate};

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB1;
pub(crate) const SIGNATURE: u8 = 0x44;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Date {
    pub(crate) value: i64,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self> {
        Ok(Self::from(
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(ValueError::InvalidDate(year, month, day))?,
        ))
    }
}

impl From<NaiveDate> for Date {
    fn from(naive_date: NaiveDate) -> Self {
        Self {
            // (seconds since epoch) / (seconds per day)
            value: (naive_date - NaiveDate::from_ymd(1970, 1, 1)).num_days(),
        }
    }
}

impl From<Date> for NaiveDate {
    fn from(date: Date) -> Self {
        NaiveDate::from_ymd(1970, 1, 1) + Duration::days(date.value)
    }
}

impl TryFrom<Value> for Date {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Date(date) => Ok(date),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

#[cfg(test)]
mod tests {
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
