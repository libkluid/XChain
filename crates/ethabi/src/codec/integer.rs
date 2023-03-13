use num_bigint::{BigInt, BigUint};
use crate::codec::sealed;
use crate::{Value, Error};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntCodec {
    name: String,
    size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UIntCodec {
    name: String,
    size: usize,
}

impl IntCodec {
    pub fn new(size: usize) -> Self {
        let name = format!("int{}", size);
        Self { name, size }
    }
}

impl UIntCodec {
    pub fn new(size: usize) -> Self {
        let name = format!("uint{}", size);
        Self { name, size }
    }
}

impl sealed::AbiType for IntCodec {
    fn name(&self) -> &str { &self.name }
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::AbiType for UIntCodec {
    fn name(&self) -> &str { &self.name }
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Encoder for IntCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let value = value.as_int()?;
        let value = value % BigInt::from(2_u32).pow(self.size as u32);

        let (_, bytes) = value.to_bytes_be();
        let bytes = std::iter::repeat(0).take(32 - bytes.len()).chain(bytes).collect();
        Ok(bytes)
    }
}

impl sealed::Decoder for IntCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];

        if frame.len() < 32 {
            return Err(Error::InvalidData)
        }

        let begin = 32 - self.size / 8;
        let value = Value::Int(BigInt::from_signed_bytes_be(&frame[begin..32]));
        Ok(value)
    }
}

impl sealed::Encoder for UIntCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let value = value.as_uint()?;
        let value = value % BigUint::from(2_u32).pow(self.size as u32);

        let bytes = value.to_bytes_be();
        let value = std::iter::repeat(0).take(32 - bytes.len()).chain(bytes).collect();
        Ok(value)
    }
}

impl sealed::Decoder for UIntCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];

        if frame.len() < 32 {
            return Err(Error::InvalidData)
        }

        let begin = 32 - self.size / 8;
        let value = Value::UInt(BigUint::from_bytes_be(&frame[begin..32]));
        Ok(value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Decoder, Encoder};

    #[test]
    fn test_uint_decoder() {
        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACE").unwrap();
        assert_eq!(
            Value::UInt(0xCE_u8.into()),
            UIntCodec::new(8).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DE0000FACE").unwrap();
        assert_eq!(
            Value::UInt(0xFACE_u16.into()),
            UIntCodec::new(16).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACE").unwrap();
        assert_eq!(
            Value::UInt(0xFEEDFACE_u32.into()),
            UIntCodec::new(32).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACEFEEDFACE").unwrap();
        assert_eq!(
            Value::UInt(0xFEEDFACEFEEDFACE_u64.into()),
            UIntCodec::new(64).decode(&bytes).unwrap()
        );

        let bytes = hex::decode("DEADC0DEDEADC0DEDEADC0DEDEADC0DEFEEDFACEFEEDFACEFEEDFACEFEEDFACE").unwrap();
        assert_eq!(
            Value::UInt(0xFEEDFACEFEEDFACEFEEDFACEFEEDFACE_u128.into()),
            UIntCodec::new(128).decode(&bytes).unwrap(),
        );
    }

    #[test]
    fn test_int_decoder() {

        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000000000FF").unwrap();
        assert_eq!(
            Value::Int(BigInt::from(-1)),
            IntCodec::new(8).decode(&bytes).unwrap(),
        );
        
        let bytes = hex::decode("000000000000000000000000000000000000000000000000000000000000FFFF").unwrap();
        assert_eq!(
            Value::Int(BigInt::from(-1)),
            IntCodec::new(16).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FFFFFFFF").unwrap();
        assert_eq!(
            Value::Int(BigInt::from(-1)),
            IntCodec::new(32).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF").unwrap();
        assert_eq!(
            Value::Int(BigInt::from(-1)),
            IntCodec::new(64).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
        assert_eq!(
            Value::Int(BigInt::from(-1)),
            IntCodec::new(128).decode(&bytes).unwrap(),
        );
    }

    #[test]
    fn test_uint_encoder() {
        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FEEDFACE").unwrap();
        assert_eq!(
            bytes,
            UIntCodec::new(256).encode(&Value::UInt(0xFEEDFACE_u32.into())).unwrap(),
        );
    }

    #[test]
    fn test_int_encoder() {
        let bytes = hex::decode("00000000000000000000000000000000000000000000000000000000FEEDFACE").unwrap();
        assert_eq!(
            bytes,
            IntCodec::new(256).encode(&Value::Int(0xFEEDFACE_u32.into())).unwrap(),
        );
    }
}
