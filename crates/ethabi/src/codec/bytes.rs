use num_traits::ToPrimitive;
use crate::codec::sealed;
use crate::{Value, Error};
use crate::codec::Encoder;
use crate::codec::UIntCodec;

pub struct FixedBytesCodec {
    name: String,
    size: usize,
}

impl FixedBytesCodec {
    pub fn new(size: usize) -> Self {
        let name = format!("bytes{}", size);
        Self { name, size }
    }
}

impl sealed::AbiType for FixedBytesCodec {
    fn name(&self) -> &str { &self.name }
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Encoder for FixedBytesCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let bytes = value.as_bytes()?;

        if bytes.len() < self.size {
            return Err(Error::InvalidData);
        }

        let mut bytes = bytes.to_vec();

        let align = 1 + self.size / 32;
        bytes.resize(32 * align, 0);

        Ok(bytes)
    }
}

impl sealed::Decoder for FixedBytesCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];

        if frame.len() >= self.size {
            Ok(Value::Bytes(frame[..self.size].to_vec()))
        } else {
            Err(Error::InvalidData)
        } 
    }
}

pub struct DynamicBytesCodec;

impl sealed::AbiType for DynamicBytesCodec {
    fn name(&self) -> &str { "bytes" }
    fn is_dynamic(&self) -> bool { true }
}

impl sealed::Encoder for DynamicBytesCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let bytes = value.as_bytes()?;

        let align = 1 + bytes.len() / 32;
        let mut buff = Vec::with_capacity(32 + 32 * align);

        let length = Value::UInt(bytes.len().into());
        buff.extend(UIntCodec::new(32).encode(&length)?);

        buff.extend(bytes);
        buff.resize(buff.capacity(), 0);

        Ok(buff)
    }
}

impl sealed::Decoder for DynamicBytesCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];
        let head = UIntCodec::new(256).decode_frame(frame, 0)?;
        let head = head.as_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        if frame.len() < length {
            return Err(Error::InvalidData)
        }

        let bytes = frame[..length].to_vec();
        Ok(Value::Bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Encoder, Decoder};

    #[test]
    fn test_fixed_bytes_encoder() {
        let bytes = hex::decode("FEEDFACE00000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            bytes,
            FixedBytesCodec::new(4).encode(&Value::Bytes(vec![0xFE, 0xED, 0xFA, 0xCE])).unwrap(),
        );
    }

    #[test]
    fn test_dynamic_bytes_encoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000008",
            "FEEDFACEFEEDFACE000000000000000000000000000000000000000000000000",
       )).unwrap();

        assert_eq!(
            bytes,
            DynamicBytesCodec.encode(&Value::Bytes(vec![0xFE, 0xED, 0xFA, 0xCE, 0xFE, 0xED, 0xFA, 0xCE])).unwrap(),
        );
    }

    #[test]
    fn test_fixed_bytes_decoder() {
        let bytes = hex::decode("FEEDFACE00000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            Value::Bytes(hex::decode("FEEDFACE").unwrap()),
            FixedBytesCodec::new(4).decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("DEADC0DEFEEDFACE000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            Value::Bytes(hex::decode("DEADC0DEFEEDFACE").unwrap()),
            FixedBytesCodec::new(8).decode(&bytes).unwrap(),
        );
    }

    #[test]
    fn test_dynamic_bytes_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000028",
            "FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
            "FEEDFACEFEEDFACEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DE",
        )).unwrap();

        assert_eq!(
            Value::Bytes(hex::decode("FEEDFACE".repeat(10)).unwrap()),
            DynamicBytesCodec.decode(&bytes).unwrap(),
        );
    }
}
