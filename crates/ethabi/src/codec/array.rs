use num_traits::ToPrimitive;
use crate::codec::sealed;
use crate::Value;
use crate::codec::{Codec, Encoder};
use crate::codec::UIntCodec;

pub struct FixedArrayCodec {
    size: usize,
    codec: Box<dyn Codec>
}

impl FixedArrayCodec {
    pub fn new(size: usize, codec: Box<dyn Codec>) -> Self {
        Self { size, codec }
    }
}

impl sealed::AbiType for FixedArrayCodec {
    fn is_dynamic(&self) -> bool { self.codec.is_dynamic() }
}

impl sealed::Encoder for FixedArrayCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let values = value.as_array().expect("Expected array");
        assert!(values.len() == self.size);

        let mut bytes = Vec::new();

        for value in values {
            bytes.extend(self.codec.encode(value));
        }

        bytes
    }
}

impl sealed::Decoder for FixedArrayCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let values: Vec<Value> = (0..self.size).map(|index| {
            self.codec.decode_frame(&frame, 32 * index)
        }).collect();
        Value::Array(values)
    }
}


pub struct DynamicArrayCodec {
    codec: Box<dyn Codec>,
}

impl DynamicArrayCodec {
    pub fn new(codec: Box<dyn Codec>) -> Self {
        Self { codec }
    }
}

impl sealed::AbiType for DynamicArrayCodec {
    fn is_dynamic(&self) -> bool { true }
}

impl sealed::Encoder for DynamicArrayCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let values = value.as_array().expect("Expected array");

        let mut bytes = Vec::new();

        let length = Value::UInt(values.len().into());
        bytes.extend(UIntCodec::new(256).encode(&length));

        for value in values {
            bytes.extend(self.codec.encode(value));
        }

        bytes
    }
}


impl sealed::Decoder for DynamicArrayCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let head = UIntCodec::new(256).decode_frame(frame, 0);
        let head = head.as_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        let values: Vec<Value> = (0..length).map(|index| {
            self.codec.decode_frame(&frame, 32 * index)
        }).collect();
        Value::Array(values)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Encoder, Decoder};

    #[test]
    fn test_dynamic_array_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        let inner = UIntCodec::new(8);
        let codec = DynamicArrayCodec {
            codec: Box::new(inner),
        };
        assert_eq!(codec.decode(&bytes), Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
            Value::UInt(3_u8.into()),
            Value::UInt(4_u8.into()),
        ]));
    }

    #[test]
    fn test_fixed_array_encoder() {
        let codec = FixedArrayCodec::new(
            2,
            Box::new(UIntCodec::new(256)),
        );

        let value = Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
        ]);

        let bytes = codec.encode(&value);
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
        let codec = DynamicArrayCodec::new(
            Box::new(UIntCodec::new(256)),
        );

        let value = Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
        ]);

        let bytes = codec.encode(&value);
        assert_eq!(
            bytes,
            hex::decode(concat!(
                "0000000000000000000000000000000000000000000000000000000000000002",
                "0000000000000000000000000000000000000000000000000000000000000001",
                "0000000000000000000000000000000000000000000000000000000000000002",
            )).unwrap()
        )
    }

    #[test]
    fn test_fixed_array_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
        )).unwrap();

        let inner = UIntCodec::new(8);
        let codec = FixedArrayCodec::new(2, Box::new(inner));

        assert_eq!(codec.decode(&bytes), Value::Array(vec![
            Value::UInt(1_u8.into()),
            Value::UInt(2_u8.into()),
        ]));
    }

}
