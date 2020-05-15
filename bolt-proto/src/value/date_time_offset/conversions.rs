use std::convert::TryFrom;

use chrono::{DateTime, FixedOffset, Offset, TimeZone, Timelike};

use crate::error::*;
use crate::value::DateTimeOffset;
use crate::Value;

impl<T: TimeZone> From<DateTime<T>> for DateTimeOffset {
    fn from(date_time: DateTime<T>) -> Self {
        Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            offset_seconds: date_time.offset().fix().local_minus_utc(),
        }
    }
}

impl TryFrom<Value> for DateTime<FixedOffset> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::DateTimeOffset(date_time_offset) => Ok(FixedOffset::east(
                date_time_offset.offset_seconds,
            )
            .timestamp(
                date_time_offset.epoch_seconds,
                date_time_offset.nanos as u32,
            )),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
