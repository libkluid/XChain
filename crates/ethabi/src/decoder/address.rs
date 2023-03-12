use crate::Value;
use crate::decoder::sealed;

pub struct AddressDecoder;

impl sealed::Decoder for AddressDecoder {
    fn is_dynamic(&self) -> bool { false }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        Value::Address(hex::encode(&bytes[offset..][12..32]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

    #[test]
    fn test_address_decoder() {
        let bytes = hex::decode(concat!(
            "000000000000000000000000FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
        )).unwrap();
        assert_eq!(AddressDecoder.decode(&bytes), Value::Address("feedfacefeedfacefeedfacefeedfacefeedface".to_string()));
    }
}
