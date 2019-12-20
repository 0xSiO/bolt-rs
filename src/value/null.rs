use crate::value::{MarkerResult, Value, ValueError};

struct Null;

impl Value for Null {
    fn get_marker(&self) -> MarkerResult {
        Ok(0xC0)
    }
}
