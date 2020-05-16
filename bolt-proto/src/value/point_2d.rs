use bolt_proto_derive::*;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x58;

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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::serialization::*;
    use crate::value::float::MARKER as FLOAT_MARKER;

    use super::*;

    fn get_point() -> Point2D {
        Point2D::new(120, 5_421_394.569_325_1, 1.9287)
    }

    #[test]
    fn get_marker() {
        let point = get_point();
        assert_eq!(point.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn try_into_bytes() {
        let point = get_point();
        assert_eq!(
            point.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                0x78,
                FLOAT_MARKER,
                0x41,
                0x54,
                0xAE,
                0x54,
                0xA4,
                0x6F,
                0xD2,
                0x8B,
                FLOAT_MARKER,
                0x3F,
                0xFE,
                0xDB,
                0xF4,
                0x87,
                0xFC,
                0xB9,
                0x24
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let point = get_point();
        let point_bytes = &[
            0x78,
            FLOAT_MARKER,
            0x41,
            0x54,
            0xAE,
            0x54,
            0xA4,
            0x6F,
            0xD2,
            0x8B,
            FLOAT_MARKER,
            0x3F,
            0xFE,
            0xDB,
            0xF4,
            0x87,
            0xFC,
            0xB9,
            0x24,
        ];
        assert_eq!(
            Point2D::try_from(Arc::new(Mutex::new(Bytes::from_static(point_bytes)))).unwrap(),
            point
        );
    }
}
