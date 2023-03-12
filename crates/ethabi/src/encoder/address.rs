use crate::Value;
use crate::encoder::sealed;

pub struct AddressEncoder;

impl sealed::Encoder for AddressEncoder {
    fn is_dynamic(&self) -> bool { false }

    fn encode_frame(&self, value: &Value) -> Vec<u8> {
        let address = value.as_address().expect("Expected address");
        assert!(address.len() == 40);

        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&[0u8; 12]);
        
        bytes.extend_from_slice(&hex::decode(address).unwrap());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::sealed::Encoder;

    #[test]
    fn test_address_encoder() {
        let bytes = hex::decode(concat!(
            "000000000000000000000000FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
        )).unwrap();
        assert_eq!(AddressEncoder.encode_frame(&Value::Address("feedfacefeedfacefeedfacefeedfacefeedface".to_string())), bytes);
    }
}
