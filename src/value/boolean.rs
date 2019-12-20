use crate::messaging::{MarkerResult, Serialize};

const MARKER_FALSE: u8 = 0xC2;
const MARKER_TRUE: u8 = 0xC3;

pub struct Boolean {
    value: bool
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl Serialize for Boolean {
    fn get_marker(&self) -> MarkerResult {
        if self.value {
            Ok(MARKER_TRUE)
        } else {
            Ok(MARKER_FALSE)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messaging::Serialize;

    use super::{Boolean, MARKER_FALSE, MARKER_TRUE};

    #[test]
    fn is_valid() {
        assert_eq!(Boolean::from(false).get_marker().unwrap(), MARKER_FALSE);
        assert_eq!(Boolean::from(true).get_marker().unwrap(), MARKER_TRUE)
    }
}
