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

impl From<NaiveDate> for Date {
    fn from(naive_date: NaiveDate) -> Self {
        Self {
            // (seconds since epoch) / (seconds per day)
            value: (NaiveDate::from_ymd(1970, 1, 1) - naive_date).num_days(),
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
            Value::Date(date) => Ok(Date::from(date)),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
