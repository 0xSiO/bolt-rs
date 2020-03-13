use crate::value::ByteArray;

impl From<&[u8]> for ByteArray {
    fn from(value: &[u8]) -> Self {
        Self {
            value: Vec::from(value),
        }
    }
}

impl From<Vec<u8>> for ByteArray {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

impl From<ByteArray> for Vec<u8> {
    fn from(byte_array: ByteArray) -> Self {
        byte_array.value
    }
}

// We don't need TryFrom<Value> for ByteArray since it can be converted directly into a Vec
// impl_try_from_value!(ByteArray, Bytes);
