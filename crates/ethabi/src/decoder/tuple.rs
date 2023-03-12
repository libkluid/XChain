use crate::Value;
use crate::decoder::{sealed, Decoder};
use crate::decoder::HeadTailDecoder;

pub struct TupleDecoder {
    decoders: Vec<Box<dyn Decoder>>,
}

impl TupleDecoder {
    pub fn new(decoders: Vec<Box<dyn Decoder>>) -> Self {
        let decoders = decoders.into_iter().map(|decoder| {
            if decoder.is_dynamic() {
                Box::new(HeadTailDecoder::new(decoder)) as Box<dyn Decoder>
            } else {
                decoder
            }
        }).collect();
        Self { decoders }
    }
}

impl sealed::Decoder for TupleDecoder {
    fn is_dynamic(&self) -> bool {
        self.decoders.iter().any(|decoder| decoder.is_dynamic())
    }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let frame = &bytes[offset..];
        let values: Vec<Value> = self.decoders.iter().enumerate().map(|(index, decoder)| {
            decoder.decode_frame(&frame, 32 * index)
        }).collect();
        Value::Tuple(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;
    use crate::decoder::{
        UIntDecoder,
        StringDecoder,
        DynamicArrayDecoder,
    };

    #[test]
    fn test_static_tuple_decoder() {
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "6162630000000000000000000000000000000000000000000000000000000000",
        )).unwrap();

        let uintdecoder = UIntDecoder::new(256);
        let stringdecoder = StringDecoder;
        let decoder = TupleDecoder::new(vec![Box::new(uintdecoder), Box::new(stringdecoder)]);
        assert_eq!(
            decoder.decode(&bytes),
            Value::Tuple(vec![
                Value::UInt(1_u8.into()),
                Value::String("abc".to_string())
            ])
        );
    }

    #[test]
    fn test_dynamic_tuple_decoder() {
        // (uint, (uint, uint[]))
        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000006",
        )).unwrap();

        let uint_decoder = UIntDecoder::new(256);
        let array_decoder = DynamicArrayDecoder::new(Box::new(uint_decoder));
        let tuple_decoder = TupleDecoder::new(vec![Box::new(uint_decoder), Box::new(array_decoder)]);
        let tuple_decoder = TupleDecoder::new(vec![Box::new(uint_decoder), Box::new(tuple_decoder)]);

        assert_eq!(
            tuple_decoder.decode(&bytes),
            Value::Tuple(vec![
                Value::UInt(1_u8.into()),
                Value::Tuple(vec![
                    Value::UInt(2_u8.into()),
                    Value::Array(vec![
                        Value::UInt(4_u8.into()),
                        Value::UInt(5_u8.into()),
                        Value::UInt(6_u8.into()),
                    ])
                ])
            ])
        );
    }
}
