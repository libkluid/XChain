use num_traits::ToPrimitive;
use crate::decoder::sealed;
use crate::Value;
use crate::decoder::UIntDecoder;

pub struct FixedBytesDecoder {
    size: usize,
}

impl FixedBytesDecoder {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl sealed::Decoder for FixedBytesDecoder {
    fn is_dynamic(&self) -> bool { false }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        Value::Bytes(frame[..self.size].to_vec())
    }
}

pub struct DynamicBytesDeocder;

impl sealed::Decoder for DynamicBytesDeocder {
    fn is_dynamic(&self) -> bool { true }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let head = UIntDecoder::new(256).decode_frame(frame, 0);
        let head = head.as_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        Value::Bytes(frame[..length].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

    #[test]
    fn test_fixed_bytes_decoder() {
        let bytes = hex::decode("FEEDFACE00000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(FixedBytesDecoder::new(4).decode(&bytes), Value::Bytes(hex::decode("FEEDFACE").unwrap()));

        let bytes = hex::decode("DEADC0DEFEEDFACE000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(FixedBytesDecoder::new(8).decode(&bytes), Value::Bytes(hex::decode("DEADC0DEFEEDFACE").unwrap()));
    }

    #[test]
    fn test_dynamic_bytes_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000028",
            "FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
            "FEEDFACEFEEDFACEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DEDEADC0DE",
        )).unwrap();

        assert_eq!(
            DynamicBytesDeocder.decode(&bytes),
            Value::Bytes(hex::decode("FEEDFACE".repeat(10)).unwrap()));
    }
}
