use std::convert::TryInto;
use std::hash::Hash;
use std::mem;

use bytes::{BufMut, Bytes, BytesMut};

use rust_bolt_macros::*;

use crate::message::MARKER_TINY_STRUCTURE;
use crate::serialize::{Serialize, SerializeError, SerializeResult};
use crate::structure::Structure;
use crate::value::{Map, String};

#[derive(Debug, Structure)]
pub struct Init<K, V>
where
    K: Serialize + Hash + Eq + TryInto<Bytes, Error = SerializeError>,
    V: Serialize + TryInto<Bytes, Error = SerializeError>,
{
    client_name: String,
    auth_token: Map<K, V>,
}

impl<K, V> Serialize for Init<K, V>
where
    K: Serialize + Hash + Eq + TryInto<Bytes, Error = SerializeError>,
    V: Serialize + TryInto<Bytes, Error = SerializeError>,
{
    fn get_marker(&self) -> SerializeResult<u8> {
        Ok(MARKER_TINY_STRUCTURE | 2)
    }
}

impl<K, V> TryInto<Bytes> for Init<K, V>
where
    K: Serialize + Hash + Eq + TryInto<Bytes, Error = SerializeError>,
    V: Serialize + TryInto<Bytes, Error = SerializeError>,
{
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        let marker = self.get_marker()?;
        let signature = self.get_signature();
        let client_name_bytes = self.client_name.try_into_bytes()?;
        let auth_token_bytes = self.auth_token.try_into_bytes()?;
        let mut result_bytes = BytesMut::with_capacity(
            // Marker byte, signature byte, then fields
            mem::size_of::<u8>() * 2 + client_name_bytes.len() + auth_token_bytes.len(),
        );
        result_bytes.put_u8(marker);
        result_bytes.put_u8(signature);
        result_bytes.put(client_name_bytes);
        result_bytes.put(auth_token_bytes);
        Ok(result_bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use bytes::Bytes;

    use crate::message::init::Init;
    use crate::serialize::Serialize;
    use crate::value::{Map, String};

    #[test]
    fn is_valid() {
        let msg = Init {
            client_name: String {
                value: "MyClient/1.0".to_string(),
            },
            auth_token: Map {
                value: HashMap::from_iter(
                    vec![("scheme", "basic")]
                        .into_iter()
                        .map(|(k, v)| (String::from(k.to_string()), String::from(v.to_string()))),
                ),
            },
        };
        assert_eq!(
            msg.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                0xB2, 0x01, 0x8C, 0x4D, 0x79, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x2F, 0x31, 0x2E,
                0x30, 0xA1, 0x86, 0x73, 0x63, 0x68, 0x65, 0x6D, 0x65, 0x85, 0x62, 0x61, 0x73, 0x69,
                0x63,
            ])
        );
    }
}
