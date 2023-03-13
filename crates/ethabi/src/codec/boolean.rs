use crate::codec::sealed;
use crate::Value;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BooleanCodec;

impl sealed::AbiType for BooleanCodec {
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Encoder for BooleanCodec {

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let boolean = value.as_boolean().expect("Expected boolean");
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&[0u8; 31]);
        bytes.push(*boolean as u8);
        bytes
    }
}

impl sealed::Decoder for BooleanCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let value = (&frame[..32]).iter().rev().any(|&x| x != 0);
        Value::Boolean(value)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Encoder, Decoder};

    #[test]
    fn test_boolean_encoder() {
        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(BooleanCodec.encode(&Value::Boolean(false)), bytes);

        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(BooleanCodec.encode(&Value::Boolean(true)), bytes);
    }

    #[test]
    fn test_boolean_decoder() {
        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(BooleanCodec.decode(&bytes), Value::Boolean(false));

        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(BooleanCodec.decode(&bytes), Value::Boolean(true));

        let bytes = hex::decode("8000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(BooleanCodec.decode(&bytes), Value::Boolean(true));
    }
}
