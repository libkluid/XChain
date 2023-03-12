use num_traits::ToPrimitive;
use crate::decoder::sealed;
use crate::Value;
use crate::decoder::UIntDecoder;

pub struct StringDecoder;

impl sealed::Decoder for StringDecoder {
    fn is_dynamic(&self) -> bool { true }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let head = UIntDecoder::new(256).decode_frame(frame, 0).to_uint().expect("head is uint");
        let length = head.to_usize().unwrap();

        let frame = &frame[32..];

        Value::String(String::from_utf8_lossy(&frame[..length]).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

    #[test]
    fn test_string_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000006",
            "4845594249540000000000000000000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(StringDecoder.decode(&bytes), Value::String("HEYBIT".to_string()));
    }
}
