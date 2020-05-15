use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

use crate::value::*;

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::Value {
                fn from(value: $T) -> Self {
                    Value::Integer(Integer::from(value))
                }
            }
        )*
    };
}
impl_from_int!(i8, i16, i32, i64);

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Value::Bytes(ByteArray::from(value.to_vec()))
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(ByteArray::from(value))
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        Value::List(List::from(value))
    }
}

impl<K, V> From<HashMap<K, V>> for Value
where
    K: Into<Value>,
    V: Into<Value>,
{
    fn from(value: HashMap<K, V, RandomState>) -> Self {
        Value::Map(Map::from(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(String::from(value))
    }
}

impl From<std::string::String> for Value {
    fn from(value: std::string::String) -> Self {
        Value::String(String::from(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<Node> for Value {
    fn from(value: Node) -> Self {
        Value::Node(value)
    }
}

impl From<Relationship> for Value {
    fn from(value: Relationship) -> Self {
        Value::Relationship(value)
    }
}

impl From<Path> for Value {
    fn from(value: Path) -> Self {
        Value::Path(value)
    }
}

impl From<UnboundRelationship> for Value {
    fn from(value: UnboundRelationship) -> Self {
        Value::UnboundRelationship(value)
    }
}

impl From<Date> for Value {
    fn from(value: Date) -> Self {
        Value::Date(value)
    }
}

impl From<NaiveDate> for Value {
    fn from(value: NaiveDate) -> Self {
        Value::Date(Date::from(value))
    }
}

impl From<Time> for Value {
    fn from(value: Time) -> Self {
        Value::Time(value)
    }
}

// No timezone-aware time in chrono
// chrono docs say this type is not implemented "due to the lack of usefulness and also the complexity"

impl From<DateTimeOffset> for Value {
    fn from(value: DateTimeOffset) -> Self {
        Value::DateTimeOffset(value)
    }
}

impl<T: TimeZone> From<DateTime<T>> for Value {
    fn from(value: DateTime<T>) -> Self {
        Value::DateTimeOffset(DateTimeOffset::from(value))
    }
}

impl From<DateTimeZoned> for Value {
    fn from(value: DateTimeZoned) -> Self {
        Value::DateTimeZoned(value)
    }
}

// No zoned date-time in chrono, only FixedOffset. Can't determine a zone ID from a fixed offset.

impl From<LocalTime> for Value {
    fn from(value: LocalTime) -> Self {
        Value::LocalTime(value)
    }
}

impl From<NaiveTime> for Value {
    fn from(value: NaiveTime) -> Self {
        Value::LocalTime(LocalTime::from(value))
    }
}

impl From<LocalDateTime> for Value {
    fn from(value: LocalDateTime) -> Self {
        Value::LocalDateTime(value)
    }
}

impl From<NaiveDateTime> for Value {
    fn from(value: NaiveDateTime) -> Self {
        Value::LocalDateTime(LocalDateTime::from(value))
    }
}

impl From<Duration> for Value {
    fn from(value: Duration) -> Self {
        Value::Duration(value)
    }
}

impl From<std::time::Duration> for Value {
    fn from(value: std::time::Duration) -> Self {
        Value::Duration(Duration::from(value))
    }
}

impl From<Point2D> for Value {
    fn from(value: Point2D) -> Self {
        Value::Point2D(value)
    }
}

impl From<Point3D> for Value {
    fn from(value: Point3D) -> Self {
        Value::Point3D(value)
    }
}
