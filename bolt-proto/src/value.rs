use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    hash::{Hash, Hasher},
    mem,
    ops::DerefMut,
    panic::catch_unwind,
    sync::{Arc, Mutex},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};

pub(crate) use date::Date;
pub(crate) use date_time_offset::DateTimeOffset;
pub(crate) use date_time_zoned::DateTimeZoned;
pub use duration::Duration;
pub(crate) use local_date_time::LocalDateTime;
pub(crate) use local_time::LocalTime;
pub use node::Node;
pub(crate) use null::Null;
pub use path::Path;
pub use point_2d::Point2D;
pub use point_3d::Point3D;
pub use relationship::Relationship;
pub(crate) use string::String;
pub use time::Time;
pub use unbound_relationship::UnboundRelationship;

use crate::error::*;
use crate::serialization::*;

pub(crate) mod conversions;
pub(crate) mod date;
pub(crate) mod date_time_offset;
pub(crate) mod date_time_zoned;
pub(crate) mod duration;
pub(crate) mod local_date_time;
pub(crate) mod local_time;
pub(crate) mod node;
pub(crate) mod null;
pub(crate) mod path;
pub(crate) mod point_2d;
pub(crate) mod point_3d;
pub(crate) mod relationship;
pub(crate) mod string;
pub(crate) mod time;
pub(crate) mod unbound_relationship;

pub(crate) const MARKER_FALSE: u8 = 0xC2;
pub(crate) const MARKER_TRUE: u8 = 0xC3;
pub(crate) const MARKER_INT_8: u8 = 0xC8;
pub(crate) const MARKER_INT_16: u8 = 0xC9;
pub(crate) const MARKER_INT_32: u8 = 0xCA;
pub(crate) const MARKER_INT_64: u8 = 0xCB;
pub(crate) const MARKER_FLOAT: u8 = 0xC1;
pub(crate) const MARKER_SMALL_BYTES: u8 = 0xCC;
pub(crate) const MARKER_MEDIUM_BYTES: u8 = 0xCD;
pub(crate) const MARKER_LARGE_BYTES: u8 = 0xCE;
pub(crate) const MARKER_TINY_LIST: u8 = 0x90;
pub(crate) const MARKER_SMALL_LIST: u8 = 0xD4;
pub(crate) const MARKER_MEDIUM_LIST: u8 = 0xD5;
pub(crate) const MARKER_LARGE_LIST: u8 = 0xD6;
pub(crate) const MARKER_TINY_MAP: u8 = 0xA0;
pub(crate) const MARKER_SMALL_MAP: u8 = 0xD8;
pub(crate) const MARKER_MEDIUM_MAP: u8 = 0xD9;
pub(crate) const MARKER_LARGE_MAP: u8 = 0xDA;

/// An enum that can hold values of all Bolt-compatible types.
///
/// Conversions are provided for most types, and are usually pretty intuitive ([`bool`] to
/// [`Value::Boolean`], [`i32`] to [`Value::Integer`],
/// [`HashMap`](std::collections::HashMap) to [`Value::Map`], etc.), but some types have
/// no analog in Rust, like a timezone-aware time. For such types, conversions are still
/// provided, but may feel a bit clunky (for example, you can convert a
/// `(`[`NaiveTime`](chrono::NaiveTime)`, impl `[`Offset`](chrono::Offset)`)` tuple into a
/// [`Value::Time`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // V1-compatible value types
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(HashMap<std::string::String, Value>),
    Null,
    String(String),
    Node(Node),
    Relationship(Relationship),
    Path(Path),
    UnboundRelationship(UnboundRelationship),

    // V2+-compatible value types
    Date(Date),                     // A date without a time zone, a.k.a. LocalDate
    Time(Time),                     // A time with a UTC offset, a.k.a. OffsetTime
    DateTimeOffset(DateTimeOffset), // A date-time with a UTC offset, a.k.a. OffsetDateTime
    DateTimeZoned(DateTimeZoned),   // A date-time with a time zone ID, a.k.a. ZonedDateTime
    LocalTime(LocalTime),           // A time without a time zone
    LocalDateTime(LocalDateTime),   // A date-time without a time zone
    Duration(Duration),
    Point2D(Point2D),
    Point3D(Point3D),
}

#[allow(clippy::derive_hash_xor_eq)]
// We implement Hash here despite deriving PartialEq because f64 and HashMap cannot be
// hashed and must panic
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Float(_)
            | Value::Map(_)
            | Value::Node(_)
            | Value::Relationship(_)
            | Value::UnboundRelationship(_)
            | Value::Path(_)
            | Value::Point2D(_)
            | Value::Point3D(_) => panic!("Cannot hash a {:?}", self),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(integer) => integer.hash(state),
            Value::Bytes(bytes) => bytes.hash(state),
            Value::List(list) => list.hash(state),
            Value::Null => Null.hash(state),
            Value::String(string) => string.hash(state),
            Value::Date(date) => date.hash(state),
            Value::Time(time) => time.hash(state),
            Value::DateTimeOffset(date_time_offset) => date_time_offset.hash(state),
            Value::DateTimeZoned(date_time_zoned) => date_time_zoned.hash(state),
            Value::LocalTime(local_time) => local_time.hash(state),
            Value::LocalDateTime(local_date_time) => local_date_time.hash(state),
            Value::Duration(duration) => duration.hash(state),
        }
    }
}

impl Eq for Value {
    fn assert_receiver_is_total_eq(&self) {
        if let Value::Float(_) | Value::Point2D(_) | Value::Point3D(_) = self {
            panic!("{:?} does not impl Eq", self)
        }
    }
}

impl Marker for Value {
    fn get_marker(&self) -> Result<u8> {
        match self {
            Value::Boolean(true) => Ok(MARKER_TRUE),
            Value::Boolean(false) => Ok(MARKER_FALSE),
            Value::Integer(integer) => match integer {
                -9_223_372_036_854_775_808..=-2_147_483_649
                | 2_147_483_648..=9_223_372_036_854_775_807 => Ok(MARKER_INT_64),
                -2_147_483_648..=-32_769 | 32_768..=2_147_483_647 => Ok(MARKER_INT_32),
                -32_768..=-129 | 128..=32_767 => Ok(MARKER_INT_16),
                -128..=-17 => Ok(MARKER_INT_8),
                -16..=127 => Ok(*integer as u8),
            },
            Value::Float(_) => Ok(MARKER_FLOAT),
            Value::Bytes(bytes) => match bytes.len() {
                0..=255 => Ok(MARKER_SMALL_BYTES),
                256..=65_535 => Ok(MARKER_MEDIUM_BYTES),
                65_536..=2_147_483_647 => Ok(MARKER_LARGE_BYTES),
                _ => Err(Error::ValueTooLarge(bytes.len())),
            },
            Value::List(list) => match list.len() {
                0..=15 => Ok(MARKER_TINY_LIST | list.len() as u8),
                16..=255 => Ok(MARKER_SMALL_LIST),
                256..=65_535 => Ok(MARKER_MEDIUM_LIST),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_LIST),
                len => Err(Error::ValueTooLarge(len)),
            },
            Value::Map(map) => match map.len() {
                0..=15 => Ok(MARKER_TINY_MAP | map.len() as u8),
                16..=255 => Ok(MARKER_SMALL_MAP),
                256..=65_535 => Ok(MARKER_MEDIUM_MAP),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_MAP),
                _ => Err(Error::ValueTooLarge(map.len())),
            },
            Value::Null => Null.get_marker(),
            Value::String(string) => string.get_marker(),
            Value::Node(node) => node.get_marker(),
            Value::Relationship(rel) => rel.get_marker(),
            Value::Path(path) => path.get_marker(),
            Value::UnboundRelationship(unbound_rel) => unbound_rel.get_marker(),
            Value::Date(date) => date.get_marker(),
            Value::Time(time) => time.get_marker(),
            Value::DateTimeOffset(date_time_offset) => date_time_offset.get_marker(),
            Value::DateTimeZoned(date_time_zoned) => date_time_zoned.get_marker(),
            Value::LocalTime(local_time) => local_time.get_marker(),
            Value::LocalDateTime(local_date_time) => local_date_time.get_marker(),
            Value::Duration(duration) => duration.get_marker(),
            Value::Point2D(point_2d) => point_2d.get_marker(),
            Value::Point3D(point_3d) => point_3d.get_marker(),
        }
    }
}

impl Serialize for Value {}

impl TryInto<Bytes> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        match self {
            Value::Boolean(true) => Ok(Bytes::from_static(&[MARKER_TRUE])),
            Value::Boolean(false) => Ok(Bytes::from_static(&[MARKER_FALSE])),
            Value::Integer(integer) => {
                // Worst case is 64-bit int
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<i64>());

                match integer {
                    -9_223_372_036_854_775_808..=-2_147_483_649
                    | 2_147_483_648..=9_223_372_036_854_775_807 => {
                        bytes.put_u8(MARKER_INT_64);
                        bytes.put_i64(integer);
                    }
                    -2_147_483_648..=-32_769 | 32_768..=2_147_483_647 => {
                        bytes.put_u8(MARKER_INT_32);
                        bytes.put_i32(integer as i32);
                    }
                    -32_768..=-129 | 128..=32_767 => {
                        bytes.put_u8(MARKER_INT_16);
                        bytes.put_i16(integer as i16);
                    }
                    -128..=-17 => {
                        bytes.put_u8(MARKER_INT_8);
                        bytes.put_i8(integer as i8);
                    }
                    -16..=127 => bytes.put_u8(integer as u8),
                }

                Ok(bytes.freeze())
            }
            Value::Float(f) => {
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<f64>());
                bytes.put_u8(MARKER_FLOAT);
                bytes.put_f64(f);
                Ok(bytes.freeze())
            }
            Value::Bytes(bytes) => {
                // Worst case is a large ByteArray, with marker byte, 32-bit size value, and length
                let mut buf = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + bytes.len(),
                );
                match bytes.len() {
                    0..=255 => {
                        buf.put_u8(MARKER_SMALL_BYTES);
                        buf.put_u8(bytes.len() as u8)
                    }
                    256..=65_535 => {
                        buf.put_u8(MARKER_MEDIUM_BYTES);
                        buf.put_u16(bytes.len() as u16)
                    }
                    65_536..=2_147_483_647 => {
                        buf.put_u8(MARKER_LARGE_BYTES);
                        buf.put_u32(bytes.len() as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(bytes.len())),
                }
                buf.put_slice(&bytes);
                Ok(buf.freeze())
            }
            Value::List(list) => {
                let length = list.len();

                let mut total_value_bytes: usize = 0;
                let mut value_bytes_vec: Vec<Bytes> = Vec::with_capacity(length);
                for value in list {
                    let value_bytes: Bytes = value.try_into()?;
                    total_value_bytes += value_bytes.len();
                    value_bytes_vec.push(value_bytes);
                }

                // Worst case is a large List, with marker byte, 32-bit size value, and all the
                // Value bytes
                let mut bytes = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + total_value_bytes,
                );

                match length {
                    0..=15 => bytes.put_u8(MARKER_TINY_LIST | length as u8),
                    16..=255 => {
                        bytes.put_u8(MARKER_SMALL_LIST);
                        bytes.put_u8(length as u8)
                    }
                    256..=65_535 => {
                        bytes.put_u8(MARKER_MEDIUM_LIST);
                        bytes.put_u16(length as u16);
                    }
                    65_536..=4_294_967_295 => {
                        bytes.put_u8(MARKER_LARGE_LIST);
                        bytes.put_u32(length as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(length)),
                }

                for value_bytes in value_bytes_vec {
                    bytes.put(value_bytes);
                }

                Ok(bytes.freeze())
            }
            Value::Map(map) => {
                let length = map.len();

                let mut total_value_bytes: usize = 0;
                let mut value_bytes_vec: Vec<Bytes> = Vec::with_capacity(length);
                for (key, val) in map {
                    let key_bytes: Bytes = Value::String(String::from(key)).try_into()?;
                    let val_bytes: Bytes = val.try_into()?;
                    total_value_bytes += key_bytes.len() + val_bytes.len();
                    value_bytes_vec.push(key_bytes);
                    value_bytes_vec.push(val_bytes);
                }
                // Worst case is a large Map, with marker byte, 32-bit size value, and all the
                // Value bytes
                let mut bytes = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + total_value_bytes,
                );

                match length {
                    0..=15 => bytes.put_u8(MARKER_TINY_MAP | length as u8),
                    16..=255 => {
                        bytes.put_u8(MARKER_SMALL_MAP);
                        bytes.put_u8(length as u8)
                    }
                    256..=65_535 => {
                        bytes.put_u8(MARKER_MEDIUM_MAP);
                        bytes.put_u16(length as u16);
                    }
                    65_536..=4_294_967_295 => {
                        bytes.put_u8(MARKER_LARGE_MAP);
                        bytes.put_u32(length as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(length)),
                }

                for value_bytes in value_bytes_vec {
                    bytes.put(value_bytes);
                }

                Ok(bytes.freeze())
            }
            Value::Null => Null.try_into(),
            Value::String(string) => string.try_into(),
            Value::Node(node) => node.try_into(),
            Value::Relationship(rel) => rel.try_into(),
            Value::Path(path) => path.try_into(),
            Value::UnboundRelationship(unbound_rel) => unbound_rel.try_into(),
            Value::Date(date) => date.try_into(),
            Value::Time(time) => time.try_into(),
            Value::DateTimeOffset(date_time_offset) => date_time_offset.try_into(),
            Value::DateTimeZoned(date_time_zoned) => date_time_zoned.try_into(),
            Value::LocalTime(local_time) => local_time.try_into(),
            Value::LocalDateTime(local_date_time) => local_date_time.try_into(),
            Value::Duration(duration) => duration.try_into(),
            Value::Point2D(point_2d) => point_2d.try_into(),
            Value::Point3D(point_3d) => point_3d.try_into(),
        }
    }
}

impl Deserialize for Value {}

impl TryFrom<Arc<Mutex<Bytes>>> for Value {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self> {
        catch_unwind(move || {
            let marker = input_arc.lock().unwrap()[0];
            match marker {
                null::MARKER => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Null)
                }
                MARKER_FALSE => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Boolean(false))
                }
                MARKER_TRUE => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Boolean(true))
                }
                // Tiny int
                marker if (-16..=127).contains(&(marker as i8)) => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Integer(marker as i8 as i64))
                }
                // Other int types
                MARKER_INT_8 | MARKER_INT_16 | MARKER_INT_32 | MARKER_INT_64 => {
                    let mut input_bytes = input_arc.lock().unwrap();
                    let marker = input_bytes.get_u8();

                    match marker {
                        MARKER_INT_8 => Ok(Value::Integer(input_bytes.get_i8() as i64)),
                        MARKER_INT_16 => Ok(Value::Integer(input_bytes.get_i16() as i64)),
                        MARKER_INT_32 => Ok(Value::Integer(input_bytes.get_i32() as i64)),
                        MARKER_INT_64 => Ok(Value::Integer(input_bytes.get_i64() as i64)),
                        _ => Err(DeserializationError::InvalidMarkerByte(marker).into()),
                    }
                }
                MARKER_FLOAT => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Float(input_arc.lock().unwrap().get_f64()))
                }
                MARKER_SMALL_BYTES | MARKER_MEDIUM_BYTES | MARKER_LARGE_BYTES => {
                    let mut input_bytes = input_arc.lock().unwrap();
                    let marker = input_bytes.get_u8();
                    let size = match marker {
                        MARKER_SMALL_BYTES => input_bytes.get_u8() as usize,
                        MARKER_MEDIUM_BYTES => input_bytes.get_u16() as usize,
                        MARKER_LARGE_BYTES => input_bytes.get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };
                    let mut bytes = vec![0; size];
                    input_bytes.copy_to_slice(&mut bytes);
                    Ok(Value::Bytes(bytes))
                }
                // Tiny list
                marker if (MARKER_TINY_LIST..=(MARKER_TINY_LIST | 0x0F)).contains(&marker) => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = 0x0F & marker as usize;
                    let mut list: Vec<Value> = Vec::with_capacity(size);
                    for _ in 0..size {
                        list.push(Value::try_from(Arc::clone(&input_arc))?);
                    }
                    Ok(Value::List(list))
                }
                MARKER_SMALL_LIST | MARKER_MEDIUM_LIST | MARKER_LARGE_LIST => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = match marker {
                        MARKER_SMALL_LIST => input_arc.lock().unwrap().get_u8() as usize,
                        MARKER_MEDIUM_LIST => input_arc.lock().unwrap().get_u16() as usize,
                        MARKER_LARGE_LIST => input_arc.lock().unwrap().get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };
                    let mut list: Vec<Value> = Vec::with_capacity(size);
                    for _ in 0..size {
                        list.push(Value::try_from(Arc::clone(&input_arc))?);
                    }
                    Ok(Value::List(list))
                }
                // Tiny map
                marker if (MARKER_TINY_MAP..=(MARKER_TINY_MAP | 0x0F)).contains(&marker) => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = 0x0F & marker as usize;
                    let mut hash_map: HashMap<std::string::String, Value> =
                        HashMap::with_capacity(size);
                    for _ in 0..size {
                        let key = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
                        let value = Value::try_from(Arc::clone(&input_arc))?;
                        hash_map.insert(key, value);
                    }

                    Ok(Value::Map(hash_map))
                }
                MARKER_SMALL_MAP | MARKER_MEDIUM_MAP | MARKER_LARGE_MAP => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = match marker {
                        MARKER_SMALL_MAP => input_arc.lock().unwrap().get_u8() as usize,
                        MARKER_MEDIUM_MAP => input_arc.lock().unwrap().get_u16() as usize,
                        MARKER_LARGE_MAP => input_arc.lock().unwrap().get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };

                    let mut hash_map: HashMap<std::string::String, Value> =
                        HashMap::with_capacity(size);
                    for _ in 0..size {
                        let key = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
                        let value = Value::try_from(Arc::clone(&input_arc))?;
                        hash_map.insert(key, value);
                    }

                    Ok(Value::Map(hash_map))
                }
                // Tiny string
                marker
                    if (string::MARKER_TINY..=(string::MARKER_TINY | 0x0F)).contains(&marker) =>
                {
                    Ok(Value::String(String::try_from(input_arc)?))
                }
                string::MARKER_SMALL | string::MARKER_MEDIUM | string::MARKER_LARGE => {
                    Ok(Value::String(String::try_from(input_arc)?))
                }
                // Tiny structure
                marker if (STRUCT_MARKER_TINY..=(STRUCT_MARKER_TINY | 0x0F)).contains(&marker) => {
                    deserialize_structure(input_arc)
                }
                STRUCT_MARKER_SMALL | STRUCT_MARKER_MEDIUM => deserialize_structure(input_arc),
                _ => Err(DeserializationError::InvalidMarkerByte(marker).into()),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

fn deserialize_structure(input_arc: Arc<Mutex<Bytes>>) -> Result<Value> {
    catch_unwind(move || {
        let (_marker, signature) = get_info_from_bytes(input_arc.lock().unwrap().deref_mut())?;
        match signature {
            node::SIGNATURE => Ok(Value::Node(Node::try_from(input_arc)?)),
            relationship::SIGNATURE => Ok(Value::Relationship(Relationship::try_from(input_arc)?)),
            path::SIGNATURE => Ok(Value::Path(Path::try_from(input_arc)?)),
            unbound_relationship::SIGNATURE => Ok(Value::UnboundRelationship(
                UnboundRelationship::try_from(input_arc)?,
            )),
            date::SIGNATURE => Ok(Value::Date(Date::try_from(input_arc)?)),
            time::SIGNATURE => Ok(Value::Time(Time::try_from(input_arc)?)),
            date_time_offset::SIGNATURE => {
                Ok(Value::DateTimeOffset(DateTimeOffset::try_from(input_arc)?))
            }
            date_time_zoned::SIGNATURE => {
                Ok(Value::DateTimeZoned(DateTimeZoned::try_from(input_arc)?))
            }
            local_time::SIGNATURE => Ok(Value::LocalTime(LocalTime::try_from(input_arc)?)),
            local_date_time::SIGNATURE => {
                Ok(Value::LocalDateTime(LocalDateTime::try_from(input_arc)?))
            }
            duration::SIGNATURE => Ok(Value::Duration(Duration::try_from(input_arc)?)),
            point_2d::SIGNATURE => Ok(Value::Point2D(Point2D::try_from(input_arc)?)),
            point_3d::SIGNATURE => Ok(Value::Point3D(Point3D::try_from(input_arc)?)),
            _ => Err(DeserializationError::InvalidSignatureByte(signature).into()),
        }
    })
    .map_err(|_| DeserializationError::Panicked)?
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::{self, FromIterator};

    use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};

    use super::*;

    #[test]
    fn null_from_bytes() {
        let null_bytes = Null.try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(null_bytes))).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn boolean_from_bytes() {
        let true_bytes = Bytes::from_static(&[MARKER_TRUE]);
        let false_bytes = Bytes::from_static(&[MARKER_FALSE]);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(true_bytes))).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(false_bytes))).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn tiny_integer_from_bytes() {
        let tiny = Value::Integer(110);
        let tiny_bytes = Bytes::from_static(&[110]);
        assert_eq!(&tiny.clone().try_into_bytes().unwrap(), &tiny_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_bytes))).unwrap(),
            tiny
        );
    }

    #[test]
    fn small_integer_from_bytes() {
        let small = Value::Integer(-127);
        let small_bytes = Bytes::from_static(&[0xC8, 0x81]);
        assert_eq!(&small.clone().try_into_bytes().unwrap(), &small_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_bytes))).unwrap(),
            small
        );
    }

    #[test]
    fn medium_integer_from_bytes() {
        let medium = Value::Integer(8000);
        let medium_bytes = Bytes::from_static(&[0xC9, 0x1F, 0x40]);
        assert_eq!(&medium.clone().try_into_bytes().unwrap(), &medium_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_bytes))).unwrap(),
            medium
        );
    }

    #[test]
    fn medium_negative_integer_from_bytes() {
        let medium_negative = Value::Integer(-18621);
        let medium_negative_bytes = Bytes::from_static(&[0xC9, 0xB7, 0x43]);
        assert_eq!(
            &medium_negative.clone().try_into_bytes().unwrap(),
            &medium_negative_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_negative_bytes))).unwrap(),
            medium_negative
        );
    }

    #[test]
    fn large_integer_from_bytes() {
        let large = Value::Integer(-1_000_000_000);
        let large_bytes = Bytes::from_static(&[0xCA, 0xC4, 0x65, 0x36, 0x00]);
        assert_eq!(&large.clone().try_into_bytes().unwrap(), &large_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_bytes))).unwrap(),
            large
        );
    }

    #[test]
    fn very_large_integer_from_bytes() {
        let very_large = Value::Integer(9_000_000_000_000_000_000);
        let very_large_bytes =
            Bytes::from_static(&[0xCB, 0x7C, 0xE6, 0x6C, 0x50, 0xE2, 0x84, 0x00, 0x00]);
        assert_eq!(
            &very_large.clone().try_into_bytes().unwrap(),
            &very_large_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(very_large_bytes))).unwrap(),
            very_large
        );
    }

    #[test]
    fn float_from_bytes() {
        let min = Value::Float(std::f64::MIN_POSITIVE);
        let min_bytes = min.clone().try_into_bytes().unwrap();
        let max = Value::Float(std::f64::MAX);
        let max_bytes = max.clone().try_into_bytes().unwrap();
        let e = Value::Float(std::f64::consts::E);
        let e_bytes = e.clone().try_into_bytes().unwrap();
        let pi = Value::Float(std::f64::consts::PI);
        let pi_bytes = pi.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(min_bytes))).unwrap(),
            min
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(max_bytes))).unwrap(),
            max
        );
        assert_eq!(Value::try_from(Arc::new(Mutex::new(e_bytes))).unwrap(), e);
        assert_eq!(Value::try_from(Arc::new(Mutex::new(pi_bytes))).unwrap(), pi);
    }

    #[test]
    fn byte_array_from_bytes() {
        let empty_arr = Value::Bytes(vec![]);
        let empty_arr_bytes = empty_arr.clone().try_into_bytes().unwrap();
        let small_arr = Value::Bytes(vec![1_u8; 100]);
        let small_arr_bytes = small_arr.clone().try_into_bytes().unwrap();
        let medium_arr = Value::Bytes(vec![99_u8; 1000]);
        let medium_arr_bytes = medium_arr.clone().try_into_bytes().unwrap();
        let large_arr = Value::Bytes(vec![1_u8; 100_000]);
        let large_arr_bytes = large_arr.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(empty_arr_bytes))).unwrap(),
            empty_arr
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_arr_bytes))).unwrap(),
            small_arr
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_arr_bytes))).unwrap(),
            medium_arr
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_arr_bytes))).unwrap(),
            large_arr
        );
    }

    #[test]
    fn empty_list_from_bytes() {
        let empty_list = Value::List(vec![]);
        let empty_list_bytes = Bytes::from_static(&[MARKER_TINY_LIST | 0]);
        assert_eq!(
            &empty_list.clone().try_into_bytes().unwrap(),
            &empty_list_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(empty_list_bytes))).unwrap(),
            empty_list
        );
    }

    #[test]
    fn tiny_list_from_bytes() {
        let tiny_list = Value::from(vec![100_000; 3]);
        let tiny_list_bytes = Bytes::from_static(&[
            MARKER_TINY_LIST | 3,
            MARKER_INT_32,
            0x00,
            0x01,
            0x86,
            0xA0,
            MARKER_INT_32,
            0x00,
            0x01,
            0x86,
            0xA0,
            MARKER_INT_32,
            0x00,
            0x01,
            0x86,
            0xA0,
        ]);
        assert_eq!(
            &tiny_list.clone().try_into_bytes().unwrap(),
            &tiny_list_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_list_bytes))).unwrap(),
            tiny_list
        );
    }

    #[test]
    fn small_list_from_bytes() {
        let small_list = Value::from(vec!["item"; 100]);
        let small_list_bytes = Bytes::from_iter(
            vec![MARKER_SMALL_LIST, 100].into_iter().chain(
                iter::repeat(&[string::MARKER_TINY | 4, 0x69, 0x74, 0x65, 0x6D])
                    .take(100)
                    .flatten()
                    .copied(),
            ),
        );
        assert_eq!(
            &small_list.clone().try_into_bytes().unwrap(),
            &small_list_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_list_bytes))).unwrap(),
            small_list
        );
    }

    #[test]
    fn medium_list_from_bytes() {
        let medium_list = Value::from(vec![false; 1000]);
        let medium_list_bytes = Bytes::from_iter(
            vec![MARKER_MEDIUM_LIST, 0x03, 0xE8]
                .into_iter()
                .chain(vec![MARKER_FALSE; 1000]),
        );
        assert_eq!(
            &medium_list.clone().try_into_bytes().unwrap(),
            &medium_list_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_list_bytes))).unwrap(),
            medium_list
        );
    }

    #[test]
    #[ignore]
    fn large_list_from_bytes() {
        let large_list = Value::from(vec![1_i8; 70_000]);
        let large_list_bytes = Bytes::from_iter(
            vec![MARKER_LARGE_LIST, 0x00, 0x01, 0x11, 0x70]
                .into_iter()
                .chain(vec![1; 70_000]),
        );
        assert_eq!(
            &large_list.clone().try_into_bytes().unwrap(),
            &large_list_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_list_bytes))).unwrap(),
            large_list
        );
    }

    #[test]
    fn string_from_bytes() {
        let tiny = String::from("string");
        let tiny_bytes = tiny.clone().try_into_bytes().unwrap();
        let small = String::from("string".repeat(10));
        let small_bytes = small.clone().try_into_bytes().unwrap();
        let medium = String::from("string".repeat(1000));
        let medium_bytes = medium.clone().try_into_bytes().unwrap();
        let large = String::from("string".repeat(100_000));
        let large_bytes = large.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_bytes))).unwrap(),
            Value::String(tiny)
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_bytes))).unwrap(),
            Value::String(small)
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_bytes))).unwrap(),
            Value::String(medium)
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_bytes))).unwrap(),
            Value::String(large)
        );
    }

    #[test]
    fn empty_map_from_bytes() {
        let empty_map = Value::from(HashMap::<&str, i8>::new());
        let empty_map_bytes = empty_map.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(empty_map_bytes))).unwrap(),
            empty_map
        );
    }

    #[test]
    fn tiny_map_from_bytes() {
        let tiny_map = Value::from(HashMap::<&str, i8>::from_iter(vec![("a", 1_i8)]));
        let tiny_map_bytes = tiny_map.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_map_bytes))).unwrap(),
            tiny_map
        );
    }

    #[test]
    fn small_map_from_bytes() {
        let small_map = Value::from(HashMap::<&str, i8>::from_iter(vec![
            ("a", 1_i8),
            ("b", 1_i8),
            ("c", 3_i8),
            ("d", 4_i8),
            ("e", 5_i8),
            ("f", 6_i8),
            ("g", 7_i8),
            ("h", 8_i8),
            ("i", 9_i8),
            ("j", 0_i8),
            ("k", 1_i8),
            ("l", 2_i8),
            ("m", 3_i8),
            ("n", 4_i8),
            ("o", 5_i8),
            ("p", 6_i8),
        ]));
        let small_map_bytes = small_map.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_map_bytes))).unwrap(),
            small_map
        );
    }

    fn get_node() -> Node {
        Node::new(
            24_i64,
            vec!["TestNode".to_string()],
            HashMap::from_iter(vec![
                ("key1".to_string(), -1_i8),
                ("key2".to_string(), 1_i8),
            ]),
        )
    }

    fn get_rel() -> Relationship {
        Relationship::new(
            24_i64,
            32_i64,
            128_i64,
            "TestRel".to_string(),
            HashMap::from_iter(vec![
                ("key1".to_string(), -2_i8),
                ("key2".to_string(), 2_i8),
            ]),
        )
    }

    fn get_unbound_rel() -> UnboundRelationship {
        UnboundRelationship::new(
            128_i64,
            "TestRel".to_string(),
            HashMap::from_iter(vec![
                ("key1".to_string(), -2_i8),
                ("key2".to_string(), 2_i8),
            ]),
        )
    }

    #[test]
    fn node_from_bytes() {
        let node_bytes: Bytes = get_node().try_into_bytes().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(node_bytes))).unwrap(),
            Value::Node(get_node())
        );
    }

    #[test]
    fn relationship_from_bytes() {
        let rel_bytes: Bytes = get_rel().try_into_bytes().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(rel_bytes))).unwrap(),
            Value::Relationship(get_rel())
        );
    }

    #[test]
    fn path_from_bytes() {
        let path = Path::new(vec![get_node()], vec![get_unbound_rel()], vec![100, 101]);
        let path_bytes: Bytes = path.clone().try_into_bytes().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(path_bytes))).unwrap(),
            Value::Path(path)
        );
    }

    #[test]
    fn unbound_relationship_from_bytes() {
        let unbound_rel_bytes: Bytes = get_unbound_rel().try_into_bytes().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(unbound_rel_bytes))).unwrap(),
            Value::UnboundRelationship(get_unbound_rel())
        );
    }

    #[test]
    fn date_from_bytes() {
        let christmas = Date::from(NaiveDate::from_ymd(2020, 12, 25));
        let christmas_bytes: Bytes = christmas.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(christmas_bytes))).unwrap(),
            Value::Date(christmas)
        );
    }

    #[test]
    fn time_from_bytes() {
        let midnight_utc = Time::from((NaiveTime::from_hms_nano(0, 0, 0, 0), Utc));
        let midnight_utc_bytes = midnight_utc.clone().try_into_bytes().unwrap();
        let about_four_pm_pacific = Time::from((
            NaiveTime::from_hms_nano(16, 4, 35, 235),
            FixedOffset::east(-8 * 3600),
        ));
        let about_four_pm_pacific_bytes = about_four_pm_pacific.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(midnight_utc_bytes))).unwrap(),
            Value::Time(midnight_utc)
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(about_four_pm_pacific_bytes))).unwrap(),
            Value::Time(about_four_pm_pacific)
        );
    }

    #[test]
    fn date_time_offset_from_bytes() {
        let date_time = DateTimeOffset::from(
            FixedOffset::east(-5 * 3600)
                .from_utc_datetime(&NaiveDate::from_ymd(2050, 12, 31).and_hms_nano(23, 59, 59, 10)),
        );
        let date_time_bytes = date_time.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(date_time_bytes))).unwrap(),
            Value::DateTimeOffset(date_time)
        );
    }

    #[test]
    fn date_time_zoned_from_bytes() {
        let date_time = DateTimeZoned::from((
            NaiveDate::from_ymd(2030, 8, 3).and_hms_milli(14, 30, 1, 2),
            chrono_tz::Asia::Ulaanbaatar,
        ));
        let date_time_bytes = date_time.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(date_time_bytes))).unwrap(),
            Value::DateTimeZoned(date_time)
        );
    }

    #[test]
    fn local_time_from_bytes() {
        let local_time = LocalTime::from(NaiveTime::from_hms_nano(23, 59, 59, 999));
        let local_time_bytes = local_time.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(local_time_bytes))).unwrap(),
            Value::LocalTime(local_time)
        );
    }

    #[test]
    fn local_date_time_from_bytes() {
        let local_date_time =
            LocalDateTime::from(NaiveDate::from_ymd(1999, 2, 27).and_hms_nano(1, 0, 0, 9999));
        let local_date_time_bytes = local_date_time.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(local_date_time_bytes))).unwrap(),
            Value::LocalDateTime(local_date_time)
        );
    }

    #[test]
    fn duration_from_bytes() {
        let duration = Duration::new(9876, 12345, 65332, 23435);
        let duration_bytes = duration.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(duration_bytes))).unwrap(),
            Value::Duration(duration)
        );
    }

    #[test]
    fn point_from_bytes() {
        let point2d = Point2D::new(9876, 12.312_345, 134_564.123_567_543);
        let point2d_bytes = point2d.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(point2d_bytes))).unwrap(),
            Value::Point2D(point2d)
        );

        let point3d = Point3D::new(249, 543.598_387, 2_945_732_849.293_85, 45_438.874_385);
        let point3d_bytes = point3d.clone().try_into_bytes().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(point3d_bytes))).unwrap(),
            Value::Point3D(point3d)
        );
    }

    #[test]
    #[ignore]
    fn value_size() {
        use std::mem::size_of;
        println!("Date: {} bytes", size_of::<Date>());
        println!("DateTimeOffset: {} bytes", size_of::<DateTimeOffset>());
        println!("DateTimeZoned: {} bytes", size_of::<DateTimeZoned>());
        println!("Duration: {} bytes", size_of::<Duration>());
        println!("LocalDateTime: {} bytes", size_of::<LocalDateTime>());
        println!("LocalTime: {} bytes", size_of::<LocalTime>());
        println!("Node: {} bytes", size_of::<Node>());
        println!("Null: {} bytes", size_of::<Null>());
        println!("Path: {} bytes", size_of::<Path>());
        println!("Point2D: {} bytes", size_of::<Point2D>());
        println!("Point3D: {} bytes", size_of::<Point3D>());
        println!("Relationship: {} bytes", size_of::<Relationship>());
        println!("String: {} bytes", size_of::<String>());
        println!("Time: {} bytes", size_of::<Time>());
        println!(
            "UnboundRelationship: {} bytes",
            size_of::<UnboundRelationship>()
        );
        println!("Value: {} bytes", size_of::<Value>())
    }
}
