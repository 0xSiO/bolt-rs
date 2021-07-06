use bolt_proto_derive::*;

use crate::value::SIGNATURE_POINT_3D;

#[bolt_structure(SIGNATURE_POINT_3D)]
#[derive(Debug, Clone, PartialEq)]
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
