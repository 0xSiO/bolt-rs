use crate::value::{MarkerResult, Value, ValueError};

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
