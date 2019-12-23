use rust_bolt_macros::*;

use crate::bolt::value::BoltValue;

pub const SIGNATURE: u8 = 0x70;

#[derive(Debug, Structure)]
pub struct BoltSuccess {
    metadata: BoltValue,
}

#[cfg(test)]
mod tests {
    //    #[test]
    //    fn try_from_bytes() {
    //        todo!()
    //    }
}
