use bolt_proto_derive::*;

use crate::impl_try_from_value;

pub(crate) const MARKER: u8 = 0xB4;
pub(crate) const SIGNATURE: u8 = 0x59;

#[derive(Debug, Clone, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Point3D {
    pub(crate) srid: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Point3D {
    pub fn new(srid: i32, x: f64, y: f64, z: f64) -> Self {
        Self { srid, x, y, z }
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

    pub fn z(&self) -> f64 {
        self.z
    }
}

impl_try_from_value!(Point3D, Point3D);
