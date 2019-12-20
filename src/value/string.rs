use crate::messaging::{MarkerResult, Serialize, SerializeError};

const MARKER_TINY: u8 = 0x8;
const MARKER_SMALL: u8 = 0xD0;
const MARKER_MEDIUM: u8 = 0xD1;
const MARKER_LARGE: u8 = 0xD2;

pub struct String {
    value: std::string::String
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self { value }
    }
}

impl Serialize for String {
    fn get_marker(&self) -> MarkerResult {
        match self.value.len() {
            0..=15 => Ok(MARKER_TINY | self.value.len() as u8),
            16..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=4_294_967_295 => Ok(MARKER_LARGE),
            _ => Err(SerializeError::new(format!("String length too long: {}", self.value.len())))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messaging::Serialize;

    #[test]
    fn is_valid() {}
}
