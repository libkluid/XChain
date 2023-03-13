use crate::codec::sealed;
use crate::{Value, Error};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BooleanCodec;

impl sealed::AbiType for BooleanCodec {
    fn name(&self) -> &str { "bool" }
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Encoder for BooleanCodec {

    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let boolean = value.as_boolean()?;
        let mut buff = Vec::with_capacity(32);
        buff.extend_from_slice(&[0u8; 31]);
        buff.push(*boolean as u8);
        Ok(buff)
    }
}

impl sealed::Decoder for BooleanCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];
        if frame.len() < 32 {
            return Err(Error::InvalidData)
        } else {
            let bytes = &frame[..32];
            let value = bytes.iter().rev().any(|&x| x != 0);
            Ok(Value::Boolean(value))
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Encoder, Decoder};

    #[test]
    fn test_boolean_encoder() {
        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            bytes,
            BooleanCodec.encode(&Value::Boolean(false)).unwrap()
        );

        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(
            bytes,
            BooleanCodec.encode(&Value::Boolean(true)).unwrap()
        );
    }

    #[test]
    fn test_boolean_decoder() {
        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            Value::Boolean(false),
            BooleanCodec.decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(
            Value::Boolean(true),
            BooleanCodec.decode(&bytes).unwrap(),
        );

        let bytes = hex::decode("8000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(
            Value::Boolean(true),
            BooleanCodec.decode(&bytes).unwrap(),
        );
    }
}
