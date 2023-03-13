use num_traits::ToPrimitive;
use crate::Value;
use crate::codec::sealed;
use crate::codec::Encoder;
use crate::codec::UIntCodec;

pub struct FixedBytesCodec {
    size: usize,
}

impl FixedBytesCodec {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl sealed::AbiType for FixedBytesCodec {
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Encoder for FixedBytesCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let bytes = value.as_bytes().expect("Expected bytes");
        assert!(bytes.len() == self.size);

        let mut bytes = bytes.to_vec();

        let align = 1 + self.size / 32;
        bytes.resize(32 * align, 0);

        bytes
    }
}

impl sealed::Decoder for FixedBytesCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        Value::Bytes(frame[..self.size].to_vec())
    }
}

pub struct DynamicBytesCodec;

impl sealed::AbiType for DynamicBytesCodec {
    fn is_dynamic(&self) -> bool { true }
}

impl sealed::Encoder for DynamicBytesCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let bytes = value.as_bytes().expect("Expected bytes");

        let align = 1 + bytes.len() / 32;
        let mut buff = Vec::with_capacity(32 + 32 * align);

        let length = Value::UInt(bytes.len().into());
        buff.extend(UIntCodec::new(32).encode(&length));

        buff.extend(bytes);
        buff.resize(buff.capacity(), 0);

        buff
    }
}

impl sealed::Decoder for DynamicBytesCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let head = UIntCodec::new(256).decode_frame(frame, 0);
        let head = head.as_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        Value::Bytes(frame[..length].to_vec())
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
            FixedBytesCodec::new(4).encode(&Value::Bytes(vec![0xFE, 0xED, 0xFA, 0xCE])),
            bytes
        );
    }

    #[test]
    fn test_dynamic_bytes_encoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000008",
            "FEEDFACEFEEDFACE000000000000000000000000000000000000000000000000",
       )).unwrap();

        assert_eq!(
            DynamicBytesCodec.encode(&Value::Bytes(vec![0xFE, 0xED, 0xFA, 0xCE, 0xFE, 0xED, 0xFA, 0xCE])),
            bytes
        );
    }

    #[test]
    fn test_fixed_bytes_decoder() {
        let bytes = hex::decode("FEEDFACE00000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(FixedBytesCodec::new(4).decode(&bytes), Value::Bytes(hex::decode("FEEDFACE").unwrap()));

        let bytes = hex::decode("DEADC0DEFEEDFACE000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(FixedBytesCodec::new(8).decode(&bytes), Value::Bytes(hex::decode("DEADC0DEFEEDFACE").unwrap()));
    }

    #[test]
    fn test_dynamic_bytes_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000028",
            "FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
            "FEEDFACEFEEDFACEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DE",
        )).unwrap();

        assert_eq!(
            DynamicBytesCodec.decode(&bytes),
            Value::Bytes(hex::decode("FEEDFACE".repeat(10)).unwrap()));
    }
}
