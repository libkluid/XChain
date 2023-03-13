use num_traits::ToPrimitive;
use crate::Value;
use crate::codec::sealed;
use crate::codec::Encoder;
use crate::codec::UIntCodec;

pub struct StringCodec;

impl sealed::AbiType for StringCodec {
    fn is_dynamic(&self) -> bool { true }
}

impl sealed::Encoder for StringCodec {
    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let string = value.as_string().expect("Expected string");

        let align = 1 + string.len() / 32;
        let mut buff = Vec::with_capacity(32 + 32 * align);

        buff.extend(UIntCodec::new(256).encode(&Value::UInt(string.len().into())));
        buff.extend(string.as_bytes());
        buff.resize(buff.capacity(), 0);

        buff
    }
}

impl sealed::Decoder for StringCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];

        let head = UIntCodec::new(256).decode_frame(frame, 0);
        let head = head.as_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        Value::String(String::from_utf8_lossy(&frame[..length]).to_string())
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
            StringCodec.encode(&Value::String("HEYBIT".to_string())),
            bytes,
        )
    }

    #[test]
    fn test_string_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000006",
            "4845594249540000000000000000000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(StringCodec.decode(&bytes), Value::String("HEYBIT".to_string()));
    }
}
