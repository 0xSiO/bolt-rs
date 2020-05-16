use chrono::{NaiveTime, Offset, Timelike};

use crate::value::Time;

// No timezone-aware time in chrono, so provide separate conversion instead
impl<O: Offset> From<(NaiveTime, O)> for Time {
    fn from(pair: (NaiveTime, O)) -> Self {
        Self {
            nanos_since_midnight: pair.0.num_seconds_from_midnight() as i64 * 1_000_000_000
                + pair.0.nanosecond() as i64,
            zone_offset: pair.1.fix().local_minus_utc(),
        }
    }
}
