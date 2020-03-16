use bolt_proto_derive::*;

pub(crate) const MARKER: u8 = 0xB0;
pub(crate) const SIGNATURE: u8 = 0x0E;

#[derive(Debug, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct AckFailure;

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use crate::serialization::*;

    use super::*;

    #[test]
    fn get_marker() {
        assert_eq!(AckFailure.get_marker().unwrap(), MARKER);
    }

    #[test]
    fn get_signature() {
        assert_eq!(AckFailure.get_signature(), SIGNATURE);
    }

    #[test]
    fn try_into_bytes() {
        let msg = AckFailure;
        assert_eq!(
            msg.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER, SIGNATURE])
        );
    }

    #[test]
    fn try_from_bytes() {
        let msg = AckFailure;
        let msg_bytes = &[];
        assert_eq!(
            AckFailure::try_from(Arc::new(Mutex::new(Bytes::from_static(msg_bytes)))).unwrap(),
            msg
        );
    }
}
