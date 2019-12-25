use bolt_proto_derive::*;

pub(crate) const SIGNATURE: u8 = 0x0F;

#[derive(Debug, Signature, Marker, Serialize, Deserialize)]
pub struct Reset;

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
        let reset = Reset::try_from(Arc::new(Mutex::new(bytes)));
        assert!(reset.is_ok());
    }
}
