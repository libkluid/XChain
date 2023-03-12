use num_traits::ToPrimitive;
use crate::decoder::sealed;
use crate::Value;
use crate::decoder::Decoder;
use crate::decoder::UIntDecoder;

pub struct FixedArrayDecoder {
    size: usize,
    pub decoder: Box<dyn Decoder>,
}

impl FixedArrayDecoder {
    pub fn new(size: usize, decoder: Box<dyn Decoder>) -> Self {
        Self { size, decoder }
    }
}

impl sealed::Decoder for FixedArrayDecoder {
    fn is_dynamic(&self) -> bool { self.decoder.is_dynamic() }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let values: Vec<Value> = (0..self.size).map(|index| {
            self.decoder.decode_frame(&frame, 32 * index)
        }).collect();
        Value::Array(values)
    }
}


pub struct DynamicArrayDecoder {
    pub decoder: Box<dyn Decoder>,
}

impl DynamicArrayDecoder {
    pub fn new(decoder: Box<dyn Decoder>) -> Self {
        Self { decoder }
    }
}

impl sealed::Decoder for DynamicArrayDecoder {
    fn is_dynamic(&self) -> bool { true }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let head = UIntDecoder::new(256).decode_frame(frame, 0);
        let head = head.as_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        let values: Vec<Value> = (0..length).map(|index| {
            self.decoder.decode_frame(&frame, 32 * index)
        }).collect();
        Value::Array(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_array_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
        )).unwrap();

        let inner = UIntDecoder::new(8);
        let decoder = FixedArrayDecoder::new(2, Box::new(inner));

        assert_eq!(decoder.decode(&bytes), Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
        ]));
    }

    #[test]
    fn test_dynamic_array_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        let inner = UIntDecoder::new(8);
        let decoder = DynamicArrayDecoder {
            decoder: Box::new(inner),
        };
        assert_eq!(decoder.decode(&bytes), Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
            Value::UInt(3_u8.into()),
            Value::UInt(4_u8.into()),
        ]));
    }
}
