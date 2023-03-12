use crate::Value;
use crate::encoder::{sealed, Encoder};
use crate::encoder::UIntEncoder;

pub struct StringEncoder;

impl sealed::Encoder for StringEncoder {
    fn is_dynamic(&self) -> bool { true }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let string = value.as_string().expect("Expected string");

        let align = 1 + string.len() / 32;
        let mut buff = Vec::with_capacity(32 + 32 * align);

        buff.extend(UIntEncoder::new(256).encode(&Value::UInt(string.len().into())));
        buff.extend(string.as_bytes());
        buff.resize(buff.capacity(), 0);

        buff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::sealed::Encoder;

    #[test]
    fn test_string_encoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000006",
            "4845594249540000000000000000000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(
            StringEncoder.encode_frame(&Value::String("HEYBIT".to_string())),
            bytes,
        )
    }
}
