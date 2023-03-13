use num_bigint::{BigInt, BigUint};
use crate::codec::sealed;
use crate::Value;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IntCodec {
    size: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UIntCodec {
    size: usize,
}

impl IntCodec {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl UIntCodec {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl sealed::AbiType for IntCodec {
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::AbiType for UIntCodec {
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Encoder for IntCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let value = value.as_int().expect("Expected uint");
        let value = value % BigInt::from(2_u32).pow(self.size as u32);

        let (_, bytes) = value.to_bytes_be();
        std::iter::repeat(0).take(32 - bytes.len()).chain(bytes).collect()
    }
}

impl sealed::Decoder for IntCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let begin = 32 - self.size / 8;
        Value::Int(BigInt::from_signed_bytes_be(&frame[begin..32]))
    }
}

impl sealed::Decoder for UIntCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let begin = 32 - self.size / 8;
        Value::UInt(BigUint::from_bytes_be(&frame[begin..32]))
    }
}

impl sealed::Encoder for UIntCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let value = value.as_uint().expect("Expected uint");
        let value = value % BigUint::from(2_u32).pow(self.size as u32);

        let bytes = value.to_bytes_be();
        std::iter::repeat(0).take(32 - bytes.len()).chain(bytes).collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Decoder, Encoder};

    #[test]
    fn test_uint_decoder() {
        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACE").unwrap();
        assert_eq!(UIntCodec::new(8).decode(&bytes), Value::UInt(0xCE_u8.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DE0000FACE").unwrap();
        assert_eq!(UIntCodec::new(16).decode(&bytes), Value::UInt(0xFACE_u16.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACE").unwrap();
        assert_eq!(UIntCodec::new(32).decode(&bytes), Value::UInt(0xFEEDFACE_u32.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACEFEEDFACE").unwrap();
        assert_eq!(UIntCodec::new(64).decode(&bytes), Value::UInt(0xFEEDFACEFEEDFACE_u64.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACEFEEDFACEFEEDFACEFEEDFACE").unwrap();
        assert_eq!(UIntCodec::new(128).decode(&bytes), Value::UInt(0xFEEDFACEFEEDFACEFEEDFACEFEEDFACE_u128.into()));
    }

    #[test]
    fn test_int_decoder() {

        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000000000FF").unwrap();
        assert_eq!(IntCodec::new(8).decode(&bytes), Value::Int(BigInt::from(-1)));
        
        let bytes = hex::decode("000000000000000000000000000000000000000000000000000000000000FFFF").unwrap();
        assert_eq!(IntCodec::new(16).decode(&bytes), Value::Int(BigInt::from(-1)));

        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FFFFFFFF").unwrap();
        assert_eq!(IntCodec::new(32).decode(&bytes), Value::Int(BigInt::from(-1)));

        let bytes = hex::decode("000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF").unwrap();
        assert_eq!(IntCodec::new(64).decode(&bytes), Value::Int(BigInt::from(-1)));

        let bytes = hex::decode("00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
        assert_eq!(IntCodec::new(128).decode(&bytes), Value::Int(BigInt::from(-1)));
    }

    #[test]
    fn test_uint_encoder() {
        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FEEDFACE").unwrap();
        assert_eq!(
            UIntCodec::new(256).encode(&Value::UInt(0xFEEDFACE_u32.into())),
            bytes
        );
    }

    #[test]
    fn test_int_encoder() {
        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FEEDFACE").unwrap();
        assert_eq!(
            IntCodec::new(256).encode(&Value::Int(0xFEEDFACE_u32.into())),
            bytes
        );
    }
}
