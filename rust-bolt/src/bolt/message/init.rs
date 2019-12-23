use std::collections::HashMap;

use rust_bolt_macros::*;

use crate::bolt::value::BoltValue;

pub const SIGNATURE: u8 = 0x01;

#[derive(Debug, Structure)]
pub struct BoltInit {
    client_name: BoltValue,
    auth_token: BoltValue,
}

impl BoltInit {
    pub fn new<K, V>(client_name: &str, auth_token: HashMap<K, V>) -> BoltInit
    where
        K: Into<BoltValue>,
        V: Into<BoltValue>,
    {
        BoltInit {
            client_name: client_name.into(),
            auth_token: auth_token.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use bytes::Bytes;

    use crate::bolt::value::Marker;
    use crate::serialize::Serialize;
    use crate::structure::Structure;

    use super::*;

    fn new_msg() -> BoltInit {
        BoltInit::new(
            "MyClient/1.0",
            HashMap::from_iter(vec![("scheme", "basic")]),
        )
    }

    #[test]
    fn get_marker() {
        assert_eq!(new_msg().get_marker().unwrap(), 0xB2);
    }

    #[test]
    fn get_signature() {
        assert_eq!(new_msg().get_signature(), 0x01);
    }

    #[test]
    fn try_into_bytes() {
        assert_eq!(
            new_msg().try_into_bytes().unwrap(),
            Bytes::from_static(&[
                0xB2, 0x01, 0x8C, 0x4D, 0x79, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x2F, 0x31, 0x2E,
                0x30, 0xA1, 0x86, 0x73, 0x63, 0x68, 0x65, 0x6D, 0x65, 0x85, 0x62, 0x61, 0x73, 0x69,
                0x63,
            ])
        );
    }
}
