use num_bigint::{BigInt, BigUint};
use crate::decoder::sealed;
use crate::Value;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IntDecoder {
    size: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UIntDecoder {
    size: usize,
}

impl IntDecoder {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl UIntDecoder {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl sealed::Decoder for IntDecoder {
    fn is_dynamic(&self) -> bool { false }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let begin = 32 - self.size / 8;
        Value::Int(BigInt::from_signed_bytes_be(&frame[begin..32]))
    }
}

impl sealed::Decoder for UIntDecoder {
    fn is_dynamic(&self) -> bool { false }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let begin = 32 - self.size / 8;
        Value::UInt(BigUint::from_bytes_be(&frame[begin..32]))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

    #[test]
    fn test_uint_decoder() {
        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACE").unwrap();
        assert_eq!(UIntDecoder::new(8).decode(&bytes), Value::UInt(0xCE_u8.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DE0000FACE").unwrap();
        assert_eq!(UIntDecoder::new(16).decode(&bytes), Value::UInt(0xFACE_u16.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACE").unwrap();
        assert_eq!(UIntDecoder::new(32).decode(&bytes), Value::UInt(0xFEEDFACE_u32.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACEFEEDFACE").unwrap();
        assert_eq!(UIntDecoder::new(64).decode(&bytes), Value::UInt(0xFEEDFACEFEEDFACE_u64.into()));

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACEFEEDFACEFEEDFACEFEEDFACE").unwrap();
        assert_eq!(UIntDecoder::new(128).decode(&bytes), Value::UInt(0xFEEDFACEFEEDFACEFEEDFACEFEEDFACE_u128.into()));
    }

    #[test]
    fn test_int_decoder() {

        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000000000FF").unwrap();
        assert_eq!(IntDecoder::new(8).decode(&bytes), Value::Int(BigInt::from(-1)));
        
        let bytes = hex::decode("000000000000000000000000000000000000000000000000000000000000FFFF").unwrap();
        assert_eq!(IntDecoder::new(16).decode(&bytes), Value::Int(BigInt::from(-1)));

        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FFFFFFFF").unwrap();
        assert_eq!(IntDecoder::new(32).decode(&bytes), Value::Int(BigInt::from(-1)));

        let bytes = hex::decode("000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF").unwrap();
        assert_eq!(IntDecoder::new(64).decode(&bytes), Value::Int(BigInt::from(-1)));

        let bytes = hex::decode("00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
        assert_eq!(IntDecoder::new(128).decode(&bytes), Value::Int(BigInt::from(-1)));
    }

}
