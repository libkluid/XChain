use crate::Value;
use crate::encoder::sealed;
use crate::encoder::Encoder;
use crate::encoder::UIntEncoder;

pub struct FixedBytesEncoder {
    size: usize,
}

impl FixedBytesEncoder {
    pub fn new(size: usize) -> Self {
        FixedBytesEncoder { size }
    }
}

impl sealed::Encoder for FixedBytesEncoder {
    fn is_dynamic(&self) -> bool { false }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let bytes = value.as_bytes().expect("Expected bytes");
        assert!(bytes.len() == self.size);

        let mut bytes = bytes.to_vec();

        let align = 1 + self.size / 32;
        bytes.resize(32 * align, 0);

        bytes
    }
}

pub struct DynamicBytesEncoder;

impl sealed::Encoder for DynamicBytesEncoder {
    fn is_dynamic(&self) -> bool { true }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let bytes = value.as_bytes().expect("Expected bytes");

        let align = 1 + bytes.len() / 32;
        let mut buff = Vec::with_capacity(32 + 32 * align);

        let length = Value::UInt(bytes.len().into());
        buff.extend(UIntEncoder::new(32).encode(&length));

        buff.extend(bytes);
        buff.resize(buff.capacity(), 0);

        buff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::sealed::Encoder;

    #[test]
    fn test_fixed_bytes_encoder() {
        let bytes = hex::decode("FEEDFACE00000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            FixedBytesEncoder::new(4).encode_frame(&Value::Bytes(vec![0xFE, 0xED, 0xFA, 0xCE])),
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
            DynamicBytesEncoder.encode_frame(&Value::Bytes(vec![0xFE, 0xED, 0xFA, 0xCE, 0xFE, 0xED, 0xFA, 0xCE])),
            bytes
        );
    }
}
