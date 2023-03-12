use num_bigint::{BigInt, BigUint};

use crate::Value;
use crate::encoder::sealed;

pub struct IntEncoder {
    size: usize,
}

impl IntEncoder {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl sealed::Encoder for IntEncoder {
    fn is_dynamic(&self) -> bool { false }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let value = value.as_int().expect("Expected uint");
        let value = value % BigInt::from(2_u32).pow(self.size as u32);

        let (_, bytes) = value.to_bytes_be();
        std::iter::repeat(0).take(32 - bytes.len()).chain(bytes).collect()
    }
}

pub struct UIntEncoder {
    size: usize,
}

impl UIntEncoder {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl sealed::Encoder for UIntEncoder {
    fn is_dynamic(&self) -> bool { false }

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
    use crate::encoder::sealed::Encoder;

    #[test]
    fn test_uint_encoder() {
        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FEEDFACE").unwrap();
        assert_eq!(
            UIntEncoder::new(256).encode_frame(&Value::UInt(0xFEEDFACE_u32.into())),
            bytes
        );
    }

    #[test]
    fn test_int_encoder() {
        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FEEDFACE").unwrap();
        assert_eq!(
            IntEncoder::new(256).encode_frame(&Value::Int(0xFEEDFACE_u32.into())),
            bytes
        );
    }
}
