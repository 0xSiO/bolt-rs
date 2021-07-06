use bolt_proto_derive::*;

use crate::value::SIGNATURE_POINT_2D;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x58;

#[bolt_structure(SIGNATURE_POINT_2D)]
#[derive(Debug, Clone, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Point2D {
    pub(crate) srid: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Point2D {
    pub fn new(srid: i32, x: f64, y: f64) -> Self {
        Self { srid, x, y }
    }

    pub fn srid(&self) -> i32 {
        self.srid
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}
