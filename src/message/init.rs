use std::convert::TryInto;
use std::mem;

use bytes::{BufMut, Bytes, BytesMut};

use crate::message::MARKER_TINY_STRUCTURE;
use crate::serialize::{Serialize, SerializeError, SerializeResult};
use crate::value::{Map, String};

const SIGNATURE: u8 = 0x01;

pub struct Init {
    client_name: String,
    auth_token: Map<String, Box<dyn Serialize>>,
}

impl Serialize for Init {
    fn get_marker(&self) -> SerializeResult<u8> {
        Ok(MARKER_TINY_STRUCTURE | 2)
    }
}

impl TryInto<Bytes> for Init {
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        let marker = self.get_marker()?;
        let client_name_bytes = self.client_name.try_into_bytes()?;
        let auth_token_bytes = self.auth_token.try_into_bytes()?;
        let mut result_bytes = BytesMut::with_capacity(
            // Marker byte, signature byte, then fields
            mem::size_of::<u8>() * 2 + client_name_bytes.len() + auth_token_bytes.len(),
        );
        result_bytes.put_u8(marker);
        result_bytes.put(client_name_bytes);
        result_bytes.put(auth_token_bytes);
        Ok(result_bytes.freeze())
    }
}
