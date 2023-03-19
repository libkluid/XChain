use num_traits::ToPrimitive;
use crate::codec::sealed;
use crate::codec::{Codec, Encoder};
use crate::codec::UIntCodec;
use crate::{Value, Error};

pub struct FixedArrayCodec {
    name: String,
    size: usize,
    codec: Box<dyn Codec>
}

impl FixedArrayCodec {
    pub fn new(size: usize, codec: Box<dyn Codec>) -> Self {
        let name = format!("{}[{}]", codec.name(), size);
        Self { name, size, codec }
    }
}

impl sealed::AbiType for FixedArrayCodec {
    fn name(&self) -> &str { &self.name }
    fn is_dynamic(&self) -> bool { self.codec.is_dynamic() }
}

impl sealed::Encoder for FixedArrayCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let values = value.as_array()?;

        if values.len() != self.size {
            return Err(Error::InvalidData)
        }

        let mut buff = Vec::new();

        for value in values {
            buff.extend(self.codec.encode(value)?);
        }

        Ok(buff)
    }
}

impl sealed::Decoder for FixedArrayCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];

        let mut values = Vec::with_capacity(self.size);
        for index in 0..self.size {
            let value = if self.codec.is_dynamic() {
                let head = UIntCodec::new(256).decode_frame(frame, 32 * index)?;
                let head = head.as_uint()?;
                let frame_base = head.to_usize().unwrap();
                self.codec.decode_frame(frame, frame_base)?
            } else {
                self.codec.decode_frame(&frame, 32 * index)?
            };
            values.push(value);
        }

        Ok(Value::Array(values))
    }
}


pub struct DynamicArrayCodec {
    name: String,
    codec: Box<dyn Codec>,
}

impl DynamicArrayCodec {
    pub fn new(codec: Box<dyn Codec>) -> Self {
        let name = format!("{}[]", codec.name());
        Self { name, codec }
    }
}

impl sealed::AbiType for DynamicArrayCodec {
    fn name(&self) -> &str { &self.name }
    fn is_dynamic(&self) -> bool { true }
}

impl sealed::Encoder for DynamicArrayCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let values = value.as_array()?;

        let mut bytes = Vec::new();
        let length = Value::UInt(values.len().into());
        bytes.extend(UIntCodec::new(256).encode(&length)?);

        let tail_chunks = values.iter().map(|value| {
            self.codec.encode(value)
        }).collect::<Result<Vec<_>, _>>()?;

        if !self.codec.is_dynamic() || values.is_empty() {
            for chunk in tail_chunks {
                bytes.extend(chunk);
            }
        } else{
            let head_size = 32 * (values.len());
            let mut tail_offset = std::iter::once(0).chain(tail_chunks.iter().scan(0, |offset, chunk| {
                *offset += chunk.len();
                Some(*offset)
            }));

            for _ in 0..values.len() {
                let offset = head_size + tail_offset.next().unwrap();
                let offset = Value::UInt(offset.into());
                bytes.extend(UIntCodec::new(256).encode(&offset)?);
            }

            for data in tail_chunks {
                bytes.extend(data);
            }
        }

        Ok(bytes)
    }
}


impl sealed::Decoder for DynamicArrayCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];
        let head = UIntCodec::new(256).decode_frame(frame, 0)?;
        let head = head.as_uint()?;
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];
        let mut values = Vec::with_capacity(length);

        for index in 0..length {
            let value = if self.codec.is_dynamic() {
                let head = UIntCodec::new(256).decode_frame(frame, 32 * index)?;
                let head = head.as_uint()?;
                let frame_base = head.to_usize().unwrap();
                self.codec.decode_frame(frame, frame_base)?
            } else {
                self.codec.decode_frame(frame, 32 * index)?
            };
            values.push(value);
        }
        Ok(Value::Array(values))
    }
}


#[cfg(test)]
mod tests {
    use crate::Value;
    use crate::codec::{Encoder, Decoder, AddressCodec};
    use crate::codec::{UIntCodec, DynamicBytesCodec, TupleCodec};
    use super::{FixedArrayCodec, DynamicArrayCodec};

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
        let codec = DynamicArrayCodec::new(Box::new(inner));
        assert_eq!(
            Value::Array(vec![
                Value::UInt(1_u8.into()),
                Value::UInt(2_u8.into()),
                Value::UInt(3_u8.into()),
                Value::UInt(4_u8.into()),
            ]),
            codec.decode(&bytes).unwrap());
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

        let bytes = codec.encode(&value).unwrap();
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

        let bytes = codec.encode(&value).unwrap();
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

        assert_eq!(
            Value::Array(vec![
                Value::UInt(1_u8.into()),
                Value::UInt(2_u8.into()),
            ]),
            codec.decode(&bytes).unwrap(),
            );
    }

    #[test]
    fn test_dynamic_element_array_decoding() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000c5b6b5",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "000000000000000000000000000000000000000000000000000000000000000a",
            "0000000000000000000000000000000000000000000000000000000000000140",
            "0000000000000000000000000000000000000000000000000000000000000180",
            "00000000000000000000000000000000000000000000000000000000000001c0",
            "0000000000000000000000000000000000000000000000000000000000000200",
            "0000000000000000000000000000000000000000000000000000000000000240",
            "0000000000000000000000000000000000000000000000000000000000000280",
            "00000000000000000000000000000000000000000000000000000000000002c0",
            "0000000000000000000000000000000000000000000000000000000000000300",
            "0000000000000000000000000000000000000000000000000000000000000340",
            "0000000000000000000000000000000000000000000000000000000000000380",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "00000000000000000000000000caec2e118abc4c510440a8d1ac8565fec0180c",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000009fba0e50c6a0164edc715ac9adff9272f9ee379e",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000008a09b18bdff44acde3516847d679d4b044cdfb89",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "000000000000000000000000d86b2605e9f996d5f425c24b11ee18a72af26404",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "000000000000000000000000610e5b63b4ffb4dbfca77096678a988f6daad3e4",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000020",
            "0000000000000000000000000000000000000000000000000000000000000000",
        )).unwrap();

        let codec = TupleCodec::new(vec![
            Box::new(UIntCodec::new(256)),
            Box::new(DynamicArrayCodec::new(Box::new(DynamicBytesCodec))),
        ]);

        let value = codec.decode(&bytes).unwrap();
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::UInt(12957365_u32.into()),
                Value::Array(vec![
                    Value::Bytes(hex::decode("00000000000000000000000000caec2e118abc4c510440a8d1ac8565fec0180c").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000009fba0e50c6a0164edc715ac9adff9272f9ee379e").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000008a09b18bdff44acde3516847d679d4b044cdfb89").unwrap()),
                    Value::Bytes(hex::decode("000000000000000000000000d86b2605e9f996d5f425c24b11ee18a72af26404").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap()),
                    Value::Bytes(hex::decode("000000000000000000000000610e5b63b4ffb4dbfca77096678a988f6daad3e4").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap()),
                    Value::Bytes(hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap()),
                ])
            ])
        )
    }

    #[test]
    fn tset_dynamic_element_array_encoding() {
        let codec = TupleCodec::new(vec![
            Box::new(DynamicArrayCodec::new(
                Box::new(TupleCodec::new(vec![
                    Box::new(AddressCodec),
                    Box::new(DynamicBytesCodec),
                ]))
            ))
        ]);


        let value = Value::Tuple(vec![
            Value::Array(vec![
                Value::Tuple(vec![
                    Value::address("e1f36c7b919c9f893e2cd30b471434aa2494664a").unwrap(),
                    Value::Bytes(hex::decode(concat!(
                        "e6a43905",
                        "0000000000000000000000008e81fcc2d4a3baa0ee9044e0d7e36f59c9bba9c1",
                        "0000000000000000000000007d72b22a74a216af4a002a1095c8c707d6ec1c5f",
                    )).unwrap()),
                ])
            ])
        ]);

        let encoded = codec.encode(&value).unwrap();
        assert_eq!(
            encoded,
            hex::decode(concat!(
                "0000000000000000000000000000000000000000000000000000000000000020",
                "0000000000000000000000000000000000000000000000000000000000000001",
                "0000000000000000000000000000000000000000000000000000000000000020",
                "000000000000000000000000e1f36c7b919c9f893e2cd30b471434aa2494664a",
                "0000000000000000000000000000000000000000000000000000000000000040",
                "0000000000000000000000000000000000000000000000000000000000000044",
                "e6a439050000000000000000000000008e81fcc2d4a3baa0ee9044e0d7e36f59",
                "c9bba9c10000000000000000000000007d72b22a74a216af4a002a1095c8c707",
                "d6ec1c5f00000000000000000000000000000000000000000000000000000000",
            )).unwrap());
            
    }
}
