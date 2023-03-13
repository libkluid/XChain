use num_traits::ToPrimitive;
use crate::codec::sealed;
use crate::{Value, Error};
use crate::codec::{Codec, Encoder};
use crate::codec::UIntCodec;

pub struct TupleCodec {
    name: String,
    codecs: Vec<Box<dyn Codec>>,
}

impl TupleCodec {
    pub fn new(codecs: Vec<Box<dyn Codec>>) -> Self {
        let names = codecs.iter().map(|codec| codec.name()).collect::<Vec<_>>();
        let name = format!("({})", names.join(","));
        Self { name, codecs }
    }
}

impl sealed::AbiType for TupleCodec {
    fn name(&self) -> &str { &self.name }
    fn is_dynamic(&self) -> bool {
        self.codecs.iter().any(|codec| codec.is_dynamic())
    }
}

impl sealed::Encoder for TupleCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let values = value.as_tuple()?;
        assert!(values.len() == self.codecs.len());

        let mut head_chunks: Vec<Option<Vec<u8>>> = Vec::new();
        let mut tail_chunks = Vec::new();

        for (encoder, value) in self.codecs.iter().zip(values) {
            if encoder.is_dynamic() {
                head_chunks.push(None);
                tail_chunks.push(encoder.encode(value)?);
            } else {
                let value = encoder.encode(value)?;
                head_chunks.push(Some(value));
            }
        }

        let head_size: usize = head_chunks.iter().flat_map(|item| item.as_ref().map(|head| head.len()).or(Some(32))).sum();
        let mut tail_offset = std::iter::once(0).chain(tail_chunks.iter().scan(0, |offset, chunk| {
            *offset += chunk.len();
            Some(*offset)
        }));

        let uint_encoder = UIntCodec::new(256);

        let mut head = Vec::with_capacity(head_size);
        for head_chunk in head_chunks.into_iter() {
            if let Some(head_chunk) = head_chunk {
                head.extend_from_slice(&head_chunk);
            } else {
                let offset = head_size + tail_offset.next().unwrap();
                let value = Value::UInt(offset.into());
                head.extend_from_slice(&uint_encoder.encode(&value)?);
            }
        }

        let tail = tail_chunks.into_iter().flatten().collect();

        Ok([head, tail].concat())

    }
}

impl sealed::Decoder for TupleCodec {

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];

        let mut values = Vec::new();
        for (index, codec) in self.codecs.iter().enumerate() {
            let offset = 32 * index;

            let value = if codec.is_dynamic() {
                let head = UIntCodec::new(256).decode_frame(frame, offset)?;
                let head = head.as_uint()?;
                let frame_base = head.to_usize().unwrap();
                codec.decode_frame(frame, frame_base)?
            } else {
                codec.decode_frame(frame, 32 * index)?
            };

            values.push(value);
        }

        let value = Value::Tuple(values);
        Ok(value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Decoder, FixedBytesCodec, DynamicBytesCodec, AddressCodec};
    use crate::codec::{
        BooleanCodec,
        UIntCodec,
        StringCodec,
        DynamicArrayCodec,
    };

    #[test]
    fn test_simple_tuple_encoder() {
        let codecs: Vec<Box<dyn Codec>> = vec![
            Box::new(BooleanCodec),
            Box::new(UIntCodec::new(256)),
        ];
        let codec = TupleCodec::new(codecs);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "000000000000000000000000000000000000000000000000000000000000FFFF",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
                Value::Boolean(true),
                Value::UInt(0xFFFF_u32.into()),
            ])).unwrap()
        );
    }

    #[test]
    fn test_array_nesting_tuple_encoder() {
        let uint_encoder = UIntCodec::new(256);
        let codec: Vec<Box<dyn Codec>> = vec![
            Box::new(BooleanCodec),
            Box::new(DynamicArrayCodec::new(Box::new(uint_encoder))),
        ];
        let codec = TupleCodec::new(codec);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
                Value::Boolean(true),
                Value::Array(vec![
                    Value::UInt(3_u32.into()),
                    Value::UInt(4_u32.into()),
                ]),
            ])).unwrap(),
        );
    }

    #[test]
    fn test_complex_tuple_encoder() {
        // (uint256, (uint256, uint256[])
        let codec = TupleCodec::new(vec![
            Box::new(UIntCodec::new(256)),
            Box::new(TupleCodec::new(vec![
                Box::new(UIntCodec::new(256)),
                Box::new(DynamicArrayCodec::new(Box::new(UIntCodec::new(256)))),
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
            codec.encode(&Value::Tuple(vec![
                Value::UInt(1_u32.into()),
                Value::Tuple(vec![
                    Value::UInt(2_u32.into()),
                    Value::Array(vec![
                        Value::UInt(3_u32.into()),
                        Value::UInt(4_u32.into()),
                    ]),
                ]),
            ])).unwrap()
        );
    }

    #[test]
    fn test_more_complex_tuple_encoder() {
        // (uint,uint32[],bytes10,bytes)
        let codec = TupleCodec::new(vec![
            Box::new(UIntCodec::new(256)),
            Box::new(DynamicArrayCodec::new(Box::new(UIntCodec::new(32)))),
            Box::new(FixedBytesCodec::new(10)),
            Box::new(DynamicBytesCodec),
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
            codec.encode(&Value::Tuple(vec![
                Value::UInt(0x123_u32.into()),
                Value::Array(vec![
                    Value::UInt(0x456_u32.into()),
                    Value::UInt(0x789_u32.into()),
                ]),
                Value::Bytes("1234567890".as_bytes().to_vec()),
                Value::Bytes("Hello, world!".as_bytes().to_vec()),
            ])).unwrap()
        );
    }

    #[test]
    fn test_issue289() {
        // (uint,uint32[],bytes10,bytes)
        let codec = TupleCodec::new(vec![
            Box::new(DynamicArrayCodec::new(Box::new(AddressCodec))),
            Box::new(DynamicArrayCodec::new(Box::new(UIntCodec::new(256)))),
            Box::new(DynamicArrayCodec::new(Box::new(AddressCodec))),
            Box::new(DynamicArrayCodec::new(Box::new(UIntCodec::new(256)))),
            Box::new(DynamicArrayCodec::new(Box::new(UIntCodec::new(256)))),
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
            codec.encode(&Value::Tuple(vec![
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
            ])).unwrap()
        );
    }

    #[test]
    fn test_static_tuple_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "6162630000000000000000000000000000000000000000000000000000000000",
        )).unwrap();

        let uintdecoder = UIntCodec::new(256);
        let stringdecoder = StringCodec;
        let codec = TupleCodec::new(vec![Box::new(uintdecoder), Box::new(stringdecoder)]);
        
        assert_eq!(
            Value::Tuple(vec![
                Value::UInt(1_u8.into()),
                Value::String("abc".to_string())
            ]),
            codec.decode(&bytes).unwrap(),
        );
    }

    #[test]
    fn test_dynamic_tuple_decoder() {
        // (uint, (uint, uint[]))
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000006",
        )).unwrap();

        let uint_decoder = UIntCodec::new(256);
        let array_decoder = DynamicArrayCodec::new(Box::new(uint_decoder.clone()));
        let tuple_decoder = TupleCodec::new(vec![Box::new(uint_decoder.clone()), Box::new(array_decoder)]);
        let tuple_decoder = TupleCodec::new(vec![Box::new(uint_decoder.clone()), Box::new(tuple_decoder)]);

        assert_eq!(
            Value::Tuple(vec![
                Value::UInt(1_u8.into()),
                Value::Tuple(vec![
                    Value::UInt(2_u8.into()),
                    Value::Array(vec![
                        Value::UInt(4_u8.into()),
                        Value::UInt(5_u8.into()),
                        Value::UInt(6_u8.into()),
                    ])
                ])
            ]),
            tuple_decoder.decode(&bytes).unwrap(),
        );
    }
}
