use std::convert::TryFrom;

use chrono::{NaiveTime, Timelike};

use crate::error::*;
use crate::value::LocalTime;
use crate::Value;

impl From<NaiveTime> for LocalTime {
    fn from(naive_time: NaiveTime) -> Self {
        Self {
            nanos_since_midnight: naive_time.num_seconds_from_midnight() as i64 * 1_000_000_000
                + naive_time.nanosecond() as i64,
        }
    }
}

impl TryFrom<Value> for NaiveTime {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::LocalTime(local_time) => {
                let seconds = (local_time.nanos_since_midnight / 1_000_000_000) as u32;
                let nanos = (local_time.nanos_since_midnight % 1_000_000_000) as u32;
                // We created the LocalTime from a NaiveTime, so it can easily be converted back without worrying about
                // a panic occurring
                Ok(NaiveTime::from_num_seconds_from_midnight(seconds, nanos))
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
