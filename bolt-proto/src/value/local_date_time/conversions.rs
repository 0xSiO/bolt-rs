use chrono::{NaiveDateTime, Timelike};

use crate::impl_try_from_value;
use crate::value::LocalDateTime;

impl From<NaiveDateTime> for LocalDateTime {
    fn from(date_time: NaiveDateTime) -> Self {
        Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
        }
    }
}

impl From<LocalDateTime> for NaiveDateTime {
    fn from(local_date_time: LocalDateTime) -> Self {
        NaiveDateTime::from_timestamp(local_date_time.epoch_seconds, local_date_time.nanos as u32)
    }
}

impl_try_from_value!(LocalDateTime, LocalDateTime);
