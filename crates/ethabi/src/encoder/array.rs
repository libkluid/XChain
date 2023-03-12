use crate::Value;
use crate::encoder::{sealed, Encoder};
use crate::encoder::UIntEncoder;

pub struct FixedArrayEncoder {
    len: usize,
    encoder: Box<dyn Encoder>,
}

impl FixedArrayEncoder {
    pub fn new(len: usize, encoder: Box<dyn Encoder>) -> Self {
        FixedArrayEncoder { len, encoder }
    }
}

impl sealed::Encoder for FixedArrayEncoder {
    fn is_dynamic(&self) -> bool { self.encoder.is_dynamic() }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let values = value.as_array().expect("Expected array");
        assert!(values.len() == self.len);

        let mut bytes = Vec::new();

        for value in values {
            bytes.extend(self.encoder.encode(value));
        }

        bytes
    }
}

pub struct DynamicArrayEncoder {
    encoder: Box<dyn Encoder>,
}

impl DynamicArrayEncoder {
    pub fn new(encoder: Box<dyn Encoder>) -> Self {
        DynamicArrayEncoder { encoder }
    }
}

impl sealed::Encoder for DynamicArrayEncoder {
    fn is_dynamic(&self) -> bool { true }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let values = value.as_array().expect("Expected array");

        let mut bytes = Vec::new();

        let length = Value::UInt(values.len().into());
        bytes.extend(UIntEncoder::new(256).encode(&length));

        for value in values {
            bytes.extend(self.encoder.encode(value));
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::sealed::Encoder;

    #[test]
    fn test_fixed_array_encoder() {
        let encoder = FixedArrayEncoder::new(
            2,
            Box::new(UIntEncoder::new(256)),
        );

        let value = Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
        ]);

        let bytes = encoder.encode_frame(&value);
        assert_eq!(
            bytes,
            hex::decode(concat!(
                "0000000000000000000000000000000000000000000000000000000000000001",
                "0000000000000000000000000000000000000000000000000000000000000002",
            )).unwrap()
        )
    }

    #[test]
    fn test_dynamic_array_encoder() {
        let encoder = DynamicArrayEncoder::new(
            Box::new(UIntEncoder::new(256)),
        );

        let value = Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
        ]);

        let bytes = encoder.encode_frame(&value);
        assert_eq!(
            bytes,
            hex::decode(concat!(
                "0000000000000000000000000000000000000000000000000000000000000002",
                "0000000000000000000000000000000000000000000000000000000000000001",
                "0000000000000000000000000000000000000000000000000000000000000002",
            )).unwrap()
        )
    }
}
