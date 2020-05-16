use std::convert::TryFrom;

use chrono::{NaiveDateTime, Timelike};

use crate::error::*;
use crate::value::LocalDateTime;
use crate::Value;

impl From<NaiveDateTime> for LocalDateTime {
    fn from(date_time: NaiveDateTime) -> Self {
        Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
        }
    }
}

impl TryFrom<Value> for NaiveDateTime {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            // We created the LocalDateTime from a NaiveDateTime, so it can easily be converted back without worrying
            // about a panic occurring
            Value::LocalDateTime(local_date_time) => Ok(NaiveDateTime::from_timestamp(
                local_date_time.epoch_seconds,
                local_date_time.nanos as u32,
            )),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
