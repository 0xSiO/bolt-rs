use std::convert::TryFrom;

use chrono::{Duration, NaiveDate};

use crate::error::*;
use crate::value::Date;
use crate::Value;

impl From<NaiveDate> for Date {
    fn from(naive_date: NaiveDate) -> Self {
        Self {
            days_since_epoch: (naive_date - NaiveDate::from_ymd(1970, 1, 1)).num_days(),
        }
    }
}

impl TryFrom<Value> for NaiveDate {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Date(date) => {
                Ok(NaiveDate::from_ymd(1970, 1, 1) + Duration::days(date.days_since_epoch))
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
