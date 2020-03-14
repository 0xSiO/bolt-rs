use chrono::{Duration, NaiveDate};

use crate::impl_try_from_value;
use crate::value::Date;

impl From<NaiveDate> for Date {
    fn from(naive_date: NaiveDate) -> Self {
        Self {
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
