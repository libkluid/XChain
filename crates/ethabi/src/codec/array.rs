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
            let value = self.codec.decode_frame(&frame, 32 * index)?;
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

        for value in values {
            bytes.extend(self.codec.encode(value)?);
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
            let offset = 32 * index;
            let value = self.codec.decode_frame(&frame, offset)?;
            values.push(value);
        }
        Ok(Value::Array(values))
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

}
