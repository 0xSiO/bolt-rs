use crate::messaging::{MarkerResult, Serialize};

const MARKER: u8 = 0xC0;

pub struct Null;

impl Serialize for Null {
    fn get_marker(&self) -> MarkerResult {
        Ok(MARKER)
    }
}

#[cfg(test)]
mod tests {
    use crate::messaging::Serialize;

    use super::{MARKER, Null};

    #[test]
    fn is_valid() {
        let null = Null;
        assert_eq!(null.get_marker().unwrap(), MARKER);
    }
}
