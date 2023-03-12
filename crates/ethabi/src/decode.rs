use pest::Parser;
use crate::decoder::{
    AddressDecoder,
    BooleanDecoder,
    DynamicArrayDecoder,
    DynamicBytesDeocder,
    FixedArrayDecoder,
    FixedBytesDecoder,
    TupleDecoder,
    IntDecoder,
    UIntDecoder,
    StringDecoder,
};
use crate::grammar::{EthAbi, Rule};
use crate::Decoder;

struct EthAbiParser<'v> {
    visitor: &'v mut Visitor,
}

impl<'v> EthAbiParser<'v> {
    fn new(visitor: &'v mut Visitor) -> Self {
        Self {
            visitor,
        }
    }

    fn accept_type(&self, pair: pest::iterators::Pair<Rule>) -> Box<dyn Decoder> {
        let rule = pair.as_rule();
        let inner = pair.into_inner().next().expect("Rule::Type should have an inner: Rule::TupleType or Rule::BasicType");

        match inner.as_rule() {
            Rule::TupleType => self.accept_tuple_type(inner),
            Rule::BasicType => self.accept_basic_type(inner),
            _ => unreachable!("Rule::Type can not expand to {:?}", rule),
        }
    }

    fn accept_tuple_type(&self, pair: pest::iterators::Pair<Rule>) -> Box<dyn Decoder> {
        let rule = pair.as_rule();
        let mut inner = pair.into_inner();

        let tuple_type = match inner.next() {
            Some(pair) => pair,
            None => panic!("Rule::TupleType should have an inner: Rule::ZeroTuple or Rule::NonZeroTuple")
        };

        let tuple_decoder = match tuple_type.as_rule() {
            Rule::ZeroTuple => self.visitor.visit_zero_tuple(),
            Rule::NonZeroTuple => {
                let decoders = tuple_type.into_inner().fold(Vec::new(), |mut acc, pair| {
                    let decoder = self.accept_type(pair);
                    acc.push(decoder);
                    acc
                });
                self.visitor.visit_non_zero_tuple(decoders)
            },
            _ => unreachable!("Rule::TupleType can not expand to {:?}", rule),
        };

        match inner.next() {
            None => tuple_decoder,
            Some(array) => {
                self.accept_array(array, tuple_decoder)
            }
        }
    }

    fn accept_array(&self, pair: pest::iterators::Pair<Rule>, decoder: Box<dyn Decoder>) -> Box<dyn Decoder> {
        let rule = pair.as_rule();

        pair.into_inner().rev().fold(decoder, |decoder, pair| {
            let array_decoder: Box<dyn Decoder> = match pair.as_rule() {
                Rule::ConstArray => {
                    let digits = pair.into_inner().next().expect("Rule::ConstArray should have an inner: Rule::Digits");
                    let size = digits.as_str().parse::<usize>().expect("Rule::Digits should be a number");
                    Box::new(FixedArrayDecoder::new(size, decoder))
                }
                Rule::DynamicArray => {
                    Box::new(DynamicArrayDecoder::new(decoder))
                }
                _ => unreachable!("Rule::Array can not expand to {:?}", rule),
            };

            array_decoder
        })
    }

    fn accept_basic_type(&self, pair: pest::iterators::Pair<Rule>) -> Box<dyn Decoder> {
        let mut inner = pair.into_inner();

        let base = inner.next().expect("Rule::BasicType should have an inner: Rule::BaseType");
        let (sub, array) = if let Some(sub_or_array) = inner.next() {
            let rule = sub_or_array.as_rule();
            match rule {
                Rule::Sub => (Some(sub_or_array), inner.next()),
                Rule::Array => (None, Some(sub_or_array)),
                _ => unreachable!("Rule::BasicType can not expand to {:?}", rule),
            }
        } else {
            (None, None)
        };

        let base_name = base.as_str();
        let base_decoder: Box<dyn Decoder> = match base_name {
            "address" => Box::new(AddressDecoder),
            "bool" => Box::new(BooleanDecoder),
            "bytes" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number"));
                match size {
                    Some(size) => Box::new(FixedBytesDecoder::new(size)),
                    None => Box::new(DynamicBytesDeocder),
                }
            }
            "int" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number")).unwrap_or(256);
                if size > 256 || size % 8 != 0 {
                    panic!("int size should be 8, 16, 24, ..., 256");
                }
                Box::new(IntDecoder::new(size))
            }
            "uint" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number")).unwrap_or(256);
                if size > 256 || size % 8 != 0 {
                    panic!("int size should be 8, 16, 24, ..., 256");
                }
                Box::new(UIntDecoder::new(size))
            }
            "string" => {
                Box::new(StringDecoder)
            }
            "fixed" => unimplemented!("fixed type is not supported yet"),
            "ufixed" => unimplemented!("ufixed type is not supported yet"),
            "function" => unimplemented!("function type is not supported yet"),
            _ => unreachable!("Unknown type {:?}", base_name),
        };

        match array {
            None => base_decoder,
            Some(array) => {
                self.accept_array(array, base_decoder)
            }
        }
    }

    fn parse(&self, abi: &str) -> Box<dyn Decoder> {
        let mut pairs = EthAbi::parse(Rule::Type, abi).unwrap();
        let pair = pairs.next().expect("should have a pair");
        self.accept_type(pair)
    }
}

struct Visitor;

impl Visitor {
    fn visit_zero_tuple(&self) -> Box<dyn Decoder> {
        let decoder = TupleDecoder::new(Vec::with_capacity(0));
        Box::new(decoder)
    }

    fn visit_non_zero_tuple(&self, decoders: Vec<Box<dyn Decoder>>) -> Box<dyn Decoder> {
        let decoder = TupleDecoder::new(decoders);
        Box::new(decoder)
    }
}

pub fn decoder(types: &[String]) -> Box<dyn Decoder> {
    let mut visitor = Visitor;
    let context = EthAbiParser::new(&mut visitor);

    let decoders = types.iter().map(|t| context.parse(t)).collect::<Vec<_>>();
    let decoder = TupleDecoder::new(decoders);
    Box::new(decoder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_empty_arguments() {
        let abi = vec![];
        let decoder = decoder(&abi);
        let value = decoder.decode(&[]);
        assert_eq!(value, Value::Tuple(Vec::new()));
    }

    #[test]
    fn test_abi_parser() {
        let abi = ["uint256", "uint256[]"].into_iter().map(Into::into).collect::<Vec<_>>();
        let decoder = decoder(&abi);
        
        let value = decoder.decode(hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap().as_slice());

        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::UInt(1_u8.into()),
                Value::Array(vec![
                    Value::UInt(3_u8.into()),
                    Value::UInt(4_u8.into()),
                ]),
            ])
        )
    }

    #[test]
    fn test_issue289() {
        // github issue: https://github.com/rust-ethereum/ethabi/issues/289
        let abi = ["address[]", "uint256[]", "address[]", "uint256[]", "uint256[]"].into_iter().map(Into::into).collect::<Vec<_>>();
        let decoder = decoder(&abi);
        let bytes = hex::decode(concat!(
            "00000000000000000000000000000000000000000000000000000000000000a0",
            "0000000000000000000000000000000000000000000000000000000000000160",
            "0000000000000000000000000000000000000000000000000000000000000220",
            "0000000000000000000000000000000000000000000000000000000000000280",
            "00000000000000000000000000000000000000000000000000000000000002e0",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000002222222222222222222222222222222222222222",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000002222222222222222222222222222222222222222",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000001111111111111111111111111111111111111111",
            "0000000000000000000000002222222222222222222222222222222222222222",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000014",
            "0000000000000000000000000000000000000000000000000000000000000019",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000000"
        )).unwrap();

        let value = decoder.decode(bytes.as_slice());
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::Array(vec![
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("2222222222222222222222222222222222222222".into()),
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("2222222222222222222222222222222222222222".into()),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(2_u8.into()),
                    Value::UInt(3_u8.into()),
                    Value::UInt(4_u8.into()),
                    Value::UInt(5_u8.into()),
                ]),
                Value::Array(vec![
                    Value::Address("1111111111111111111111111111111111111111".into()),
                    Value::Address("2222222222222222222222222222222222222222".into()),
                ]),
                Value::Array(vec![
                    Value::UInt(20_u8.into()),
                    Value::UInt(25_u8.into()),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(0_u8.into()),
                ])
            ])
        )
    }
}

