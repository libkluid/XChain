use crate::Value;
use crate::codec::sealed;

pub struct AddressCodec;

impl sealed::AbiType for AddressCodec {
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Decoder for AddressCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        Value::Address(hex::encode(&bytes[offset..][12..32]))
    }
}

impl sealed::Encoder for AddressCodec {
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
    use crate::codec::{Encoder, Decoder};

    #[test]
    fn test_address_encoder() {
        let bytes = hex::decode(concat!(
            "000000000000000000000000FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
        )).unwrap();
        assert_eq!(AddressCodec.encode(&Value::Address("feedfacefeedfacefeedfacefeedfacefeedface".to_string())), bytes);
    }

    #[test]
    fn test_address_decoder() {
        let bytes = hex::decode(concat!(
            "000000000000000000000000FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
        )).unwrap();
        assert_eq!(AddressCodec.decode(&bytes), Value::Address("feedfacefeedfacefeedfacefeedfacefeedface".to_string()));
    }
}
