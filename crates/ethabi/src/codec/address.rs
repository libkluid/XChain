use crate::codec::sealed;
use crate::{Value, Error};

pub struct AddressCodec;

impl sealed::AbiType for AddressCodec {
    fn name(&self) -> &str { "address" }
    fn is_dynamic(&self) -> bool { false }
}

impl sealed::Decoder for AddressCodec {
    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error> {
        let frame = &bytes[offset..];
        if frame.len() >= 20 {
            Ok(Value::Address(frame[12..32].to_vec()))
        } else {
            Err(Error::InvalidData)
        }

    }
}

impl sealed::Encoder for AddressCodec {
    fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error> {
        let address = value.as_address()?;
        let address = strip_hex(address.as_str());

        if address.len() == 40 {
            encode_address(address)
        } else {
            Err(Error::InvalidData)
        }
    }
}

fn strip_hex(hex: &str) -> &str {
    match hex.starts_with("0x") {
        true => &hex[2..],
        false => hex,
    }
}

fn encode_address(address: &str) -> Result<Vec<u8>, Error> {
    let bytes = hex::decode(address)?;

    let mut buff = Vec::with_capacity(32);
    buff.extend_from_slice(&[0u8; 12]);
    buff.extend_from_slice(&bytes);
    Ok(buff)
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
        assert_eq!(
            bytes,
            AddressCodec.encode(&Value::address("feedfacefeedfacefeedfacefeedfacefeedface").unwrap()).unwrap()
        );
    }

    #[test]
    fn test_address_decoder() {
        let bytes = hex::decode(concat!(
            "000000000000000000000000FEEDFACEFEEDFACEFEEDFACEFEEDFACEFEEDFACE",
        )).unwrap();
        assert_eq!(
            Value::address("feedfacefeedfacefeedfacefeedfacefeedface").unwrap(),
            AddressCodec.decode(&bytes).unwrap()
        );
    }
}
