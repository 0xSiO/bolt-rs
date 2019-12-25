use bolt_proto_derive::*;

pub(crate) const SIGNATURE: u8 = 0x7E;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct Ignored;

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use super::*;

    #[test]
    fn try_from_bytes() {
        // No data needed!
        let bytes = Bytes::from_static(&[]);
        let ignored = Ignored::try_from(Arc::new(Mutex::new(bytes)));
        assert!(ignored.is_ok());
    }
}
