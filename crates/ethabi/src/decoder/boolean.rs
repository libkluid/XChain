use crate::decoder::sealed;
use crate::Value;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BooleanDecoder;

impl sealed::Decoder for BooleanDecoder {
    fn is_dynamic(&self) -> bool { false }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let value = (&frame[..32]).iter().rev().any(|&x| x != 0);
        Value::Boolean(value)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

    #[test]
    fn test_boolean_decoder() {
        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(BooleanDecoder.decode(&bytes), Value::Boolean(false));

        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(BooleanDecoder.decode(&bytes), Value::Boolean(true));

        let bytes = hex::decode("8000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(BooleanDecoder.decode(&bytes), Value::Boolean(true));
    }
}
