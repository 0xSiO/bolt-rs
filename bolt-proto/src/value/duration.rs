use bolt_proto_derive::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB4;
pub(crate) const SIGNATURE: u8 = 0x45;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Duration {
    pub(crate) months: i64,
    pub(crate) days: i64,
    pub(crate) seconds: i64,
    pub(crate) nanos: i32,
}

impl Duration {
    pub fn new(months: i64, days: i64, seconds: i64, nanos: i32) -> Self {
        Self {
            months,
            days,
            seconds,
            nanos,
        }
    }
}

#[cfg(test)]
// TODO
mod tests {}
