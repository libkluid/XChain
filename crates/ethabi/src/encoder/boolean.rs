use crate::Value;
use crate::encoder::sealed;

pub struct BooleanEncoder;

impl sealed::Encoder for BooleanEncoder {
    fn is_dynamic(&self) -> bool { false }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let boolean = value.as_boolean().expect("Expected boolean");
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&[0u8; 31]);
        bytes.push(*boolean as u8);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::sealed::Encoder;

    #[test]
    fn test_boolean_encoder() {
        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(BooleanEncoder.encode_frame(&Value::Boolean(false)), bytes);

        let bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(BooleanEncoder.encode_frame(&Value::Boolean(true)), bytes);
    }
}
