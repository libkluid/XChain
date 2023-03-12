use crate::Value;
use crate::encoder::{sealed, Encoder};
use crate::encoder::UIntEncoder;

pub struct TupleEncoder {
    encoders: Vec<Box<dyn Encoder>>,
}

impl TupleEncoder {
    pub fn new(encoders: Vec<Box<dyn Encoder>>) -> Self {
        TupleEncoder { encoders }
    }
}

impl sealed::Encoder for TupleEncoder {
    fn is_dynamic(&self) -> bool {
        self.encoders.iter().any(|encoder| encoder.is_dynamic())
    }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let values = value.as_tuple().expect("Expected tuple");
        assert!(values.len() == self.encoders.len());

        let mut head_chunks: Vec<Option<Vec<u8>>> = Vec::new();
        let mut tail_chunks = Vec::new();

        for (encoder, value) in self.encoders.iter().zip(values) {
            if encoder.is_dynamic() {
                head_chunks.push(None);
                tail_chunks.push(encoder.encode(value));
            } else {
                head_chunks.push(Some(encoder.encode(value)));
            }
        }

        let head_size: usize = head_chunks.iter().flat_map(|item| item.as_ref().map(|head| head.len()).or(Some(32))).sum();
        let mut tail_offset = std::iter::once(0).chain(tail_chunks.iter().scan(0, |offset, chunk| {
            *offset += chunk.len();
            Some(*offset)
        }));

        let uint_encoder = UIntEncoder::new(256);

        let head: Vec<u8> = head_chunks.into_iter().map(|item| {
            item.unwrap_or_else(|| {
                let offset = head_size + tail_offset.next().unwrap();
                uint_encoder.encode(&Value::UInt(offset.into()))
            })
        }).flatten().collect();

        let tail = tail_chunks.into_iter().flatten().collect();

        [head, tail].concat()

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::AddressEncoder;
    use crate::encoder::DynamicArrayEncoder;
    use crate::encoder::Encoder;
    use crate::encoder::BooleanEncoder;
    use crate::encoder::UIntEncoder;
    use crate::encoder::{DynamicBytesEncoder, FixedBytesEncoder};

    #[test]
    fn test_empty_tuple_encoder() {
        let encoder = TupleEncoder::new(vec![]);
        assert_eq!(encoder.encode(&Value::Tuple(vec![])), vec![]);
    }

    #[test]
    fn test_simple_tuple_encoder() {
        let encoders: Vec<Box<dyn Encoder>> = vec![
            Box::new(BooleanEncoder),
            Box::new(UIntEncoder::new(256)),
        ];
        let encoder = TupleEncoder::new(encoders);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "000000000000000000000000000000000000000000000000000000000000FFFF",
        )).unwrap();

        assert_eq!(
            bytes,
            encoder.encode(&Value::Tuple(vec![
            Value::Boolean(true),
            Value::UInt(0xFFFF_u32.into()),
        ])));
    }

    #[test]
    fn test_array_nesting_tuple_encoder() {
        let uint_encoder = UIntEncoder::new(256);
        let encoders: Vec<Box<dyn Encoder>> = vec![
            Box::new(BooleanEncoder),
            Box::new(DynamicArrayEncoder::new(Box::new(uint_encoder))),
        ];
        let encoder = TupleEncoder::new(encoders);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        assert_eq!(
            bytes,
            encoder.encode(&Value::Tuple(vec![
            Value::Boolean(true),
            Value::Array(vec![
                Value::UInt(3_u32.into()),
                Value::UInt(4_u32.into()),
            ]),
        ])));
    }

    #[test]
    fn test_complex_tuple_encoder() {
        // (uint256, (uint256, uint256[])
        let encoder = TupleEncoder::new(vec![
            Box::new(UIntEncoder::new(256)),
            Box::new(TupleEncoder::new(vec![
                Box::new(UIntEncoder::new(256)),
                Box::new(DynamicArrayEncoder::new(Box::new(UIntEncoder::new(256)))),
            ])),
        ]);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        assert_eq!(
            bytes,
            encoder.encode(&Value::Tuple(vec![
            Value::UInt(1_u32.into()),
            Value::Tuple(vec![
                Value::UInt(2_u32.into()),
                Value::Array(vec![
                    Value::UInt(3_u32.into()),
                    Value::UInt(4_u32.into()),
                ]),
            ]),
        ])));
    }

    #[test]
    fn test_more_complex_tuple_encoder() {
        // (uint,uint32[],bytes10,bytes)
        let encoder = TupleEncoder::new(vec![
            Box::new(UIntEncoder::new(256)),
            Box::new(DynamicArrayEncoder::new(Box::new(UIntEncoder::new(32)))),
            Box::new(FixedBytesEncoder::new(10)),
            Box::new(DynamicBytesEncoder),
        ]);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000123",
            "0000000000000000000000000000000000000000000000000000000000000080",
            "3132333435363738393000000000000000000000000000000000000000000000",
            "00000000000000000000000000000000000000000000000000000000000000e0",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000456",
            "0000000000000000000000000000000000000000000000000000000000000789",
            "000000000000000000000000000000000000000000000000000000000000000d",
            "48656c6c6f2c20776f726c642100000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(
            bytes,
            encoder.encode(&Value::Tuple(vec![
                Value::UInt(0x123_u32.into()),
                Value::Array(vec![
                    Value::UInt(0x456_u32.into()),
                    Value::UInt(0x789_u32.into()),
                ]),
                Value::Bytes("1234567890".as_bytes().to_vec()),
                Value::Bytes("Hello, world!".as_bytes().to_vec()),
            ]))
        );
    }

    #[test]
    fn test_issue289() {
        // (uint,uint32[],bytes10,bytes)
        let encoder = TupleEncoder::new(vec![
            Box::new(DynamicArrayEncoder::new(Box::new(AddressEncoder))),
            Box::new(DynamicArrayEncoder::new(Box::new(UIntEncoder::new(256)))),
            Box::new(DynamicArrayEncoder::new(Box::new(AddressEncoder))),
            Box::new(DynamicArrayEncoder::new(Box::new(UIntEncoder::new(256)))),
            Box::new(DynamicArrayEncoder::new(Box::new(UIntEncoder::new(256)))),
        ]);

        let bytes = hex::decode(concat!(
            "00000000000000000000000000000000000000000000000000000000000000a0",
            "0000000000000000000000000000000000000000000000000000000000000160",
            "0000000000000000000000000000000000000000000000000000000000000220",
            "0000000000000000000000000000000000000000000000000000000000000280",
            "00000000000000000000000000000000000000000000000000000000000002e0",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000002222222222222222222222222222222222222222",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000002222222222222222222222222222222222222222",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000002222222222222222222222222222222222222222",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000014",
            "0000000000000000000000000000000000000000000000000000000000000019",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000000"
        )).unwrap();

        assert_eq!(
            bytes,
            encoder.encode(&Value::Tuple(vec![
                Value::Array(vec![
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("2222222222222222222222222222222222222222".into()),
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("2222222222222222222222222222222222222222".into()),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(2_u8.into()),
                    Value::UInt(3_u8.into()),
                    Value::UInt(4_u8.into()),
                    Value::UInt(5_u8.into()),
                ]),
                Value::Array(vec![
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("2222222222222222222222222222222222222222".into()),
                ]),
                Value::Array(vec![
                    Value::UInt(20_u8.into()),
                    Value::UInt(25_u8.into()),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(0_u8.into()),
                ])
            ]))
        );
    }
}
