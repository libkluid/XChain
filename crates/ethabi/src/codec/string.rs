use num_traits::ToPrimitive;
use crate::codec::sealed;
use crate::{Value, Error};
use crate::codec::UIntCodec;

use crate::codec::{Encoder, Decoder};

pub struct StringCodec;

impl sealed::AbiType for StringCodec {
    fn name(&self) -> &str { "string" }
    fn is_dynamic(&self) -> bool { true }
}

impl sealed::Encoder for StringCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let string = value.as_string()?;

        let align = 1 + string.len() / 32;
        let mut buff = Vec::with_capacity(32 + 32 * align);

        let length = Value::UInt(string.len().into());
        buff.extend(UIntCodec::new(256).encode(&length)?);
        buff.extend(string.as_bytes());
        buff.resize(buff.capacity(), 0);

        Ok(buff)
    }
}

impl sealed::Decoder for StringCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];

        let head = UIntCodec::new(256).decode(frame)?;
        let head = head.as_uint()?;
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        if frame.len() < length {
            return Err(Error::InvalidData)
        }

        let value = Value::String(String::from_utf8_lossy(&frame[..length]).to_string());
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Encoder, Decoder};

    #[test]
    fn test_string_encoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000006",
            "4845594249540000000000000000000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(
            bytes,
            StringCodec.encode(&Value::String("HEYBIT".to_string())).unwrap(),
        )
    }

    #[test]
    fn test_string_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000006",
            "4845594249540000000000000000000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(
            Value::String("HEYBIT".to_string()),
            StringCodec.decode(&bytes).unwrap(),
        );
    }
}
