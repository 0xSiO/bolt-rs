use crate::messaging::value::{MarkerResult, Value, ValueError};

mod value;

struct Null;

impl Value for Null {
    fn get_marker(&self) -> MarkerResult {
        Ok(0xC0)
    }
}

struct Boolean {
    value: bool
}

impl Value for Boolean {
    fn get_marker(&self) -> MarkerResult {
        if self.value {
            Ok(0xC3)
        } else {
            Ok(0xC2)
        }
    }
}

struct String {
    value: std::string::String
}

impl Value for String {
    fn get_marker(&self) -> MarkerResult {
        match self.value.len() {
            0..=15 => Ok((0x8 | self.value.len()) as u8),
            16..=255 => Ok(0xD0),
            256..=65_535 => Ok(0xD1),
            65_536..=4_294_967_295 => Ok(0xD2),
            _ => Err(ValueError::new(format!("String length too long: {}", self.value.len())))
        }
    }
}
