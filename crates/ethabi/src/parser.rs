use pest::Parser;
use crate::codec::{
    AddressCodec,
    BooleanCodec,
    DynamicArrayCodec,
    DynamicBytesCodec,
    FixedArrayCodec,
    FixedBytesCodec,
    TupleCodec,
    IntCodec,
    UIntCodec,
    StringCodec,
};
use crate::grammar::{EthAbi, Rule};
use crate::Codec;

struct EthAbiParser<'v> {
    visitor: &'v mut Visitor,
}

impl<'v> EthAbiParser<'v> {
    fn new(visitor: &'v mut Visitor) -> Self {
        Self {
            visitor,
        }
    }

    fn accept_type(&self, pair: pest::iterators::Pair<Rule>) -> Box<dyn Codec> {
        let rule = pair.as_rule();
        let inner = pair.into_inner().next().expect("Rule::Type should have an inner: Rule::TupleType or Rule::BasicType");

        match inner.as_rule() {
            Rule::TupleType => self.accept_tuple_type(inner),
            Rule::BasicType => self.accept_basic_type(inner),
            _ => unreachable!("Rule::Type can not expand to {:?}", rule),
        }
    }

    fn accept_tuple_type(&self, pair: pest::iterators::Pair<Rule>) -> Box<dyn Codec> {
        let rule = pair.as_rule();
        let mut inner = pair.into_inner();

        let tuple_type = match inner.next() {
            Some(pair) => pair,
            None => panic!("Rule::TupleType should have an inner: Rule::ZeroTuple or Rule::NonZeroTuple")
        };

        let tuple_codec = match tuple_type.as_rule() {
            Rule::ZeroTuple => self.visitor.visit_zero_tuple(),
            Rule::NonZeroTuple => {
                let codecs = tuple_type.into_inner().fold(Vec::new(), |mut acc, pair| {
                    let codec = self.accept_type(pair);
                    acc.push(codec);
                    acc
                });
                self.visitor.visit_non_zero_tuple(codecs)
            },
            _ => unreachable!("Rule::TupleType can not expand to {:?}", rule),
        };

        match inner.next() {
            None => tuple_codec,
            Some(array) => {
                self.accept_array(array, tuple_codec)
            }
        }
    }

    fn accept_array(&self, pair: pest::iterators::Pair<Rule>, codec: Box<dyn Codec>) -> Box<dyn Codec> {
        let rule = pair.as_rule();

        pair.into_inner().rev().fold(codec, |codec, pair| {
            let array_codec: Box<dyn Codec> = match pair.as_rule() {
                Rule::ConstArray => {
                    let digits = pair.into_inner().next().expect("Rule::ConstArray should have an inner: Rule::Digits");
                    let size = digits.as_str().parse::<usize>().expect("Rule::Digits should be a number");
                    Box::new(FixedArrayCodec::new(size, codec))
                }
                Rule::DynamicArray => {
                    Box::new(DynamicArrayCodec::new(codec))
                }
                _ => unreachable!("Rule::Array can not expand to {:?}", rule),
            };

            array_codec
        })
    }

    fn accept_basic_type(&self, pair: pest::iterators::Pair<Rule>) -> Box<dyn Codec> {
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
        let base_codec: Box<dyn Codec> = match base_name {
            "address" => Box::new(AddressCodec),
            "bool" => Box::new(BooleanCodec),
            "bytes" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number"));
                match size {
                    Some(size) => Box::new(FixedBytesCodec::new(size)),
                    None => Box::new(DynamicBytesCodec),
                }
            }
            "int" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number")).unwrap_or(256);
                if size > 256 || size % 8 != 0 {
                    panic!("int size should be 8, 16, 24, ..., 256");
                }
                Box::new(IntCodec::new(size))
            }
            "uint" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number")).unwrap_or(256);
                if size > 256 || size % 8 != 0 {
                    panic!("int size should be 8, 16, 24, ..., 256");
                }
                Box::new(UIntCodec::new(size))
            }
            "string" => {
                Box::new(StringCodec)
            }
            "fixed" => unimplemented!("fixed type is not supported yet"),
            "ufixed" => unimplemented!("ufixed type is not supported yet"),
            "function" => unimplemented!("function type is not supported yet"),
            _ => unreachable!("Unknown type {:?}", base_name),
        };

        match array {
            None => base_codec,
            Some(array) => {
                self.accept_array(array, base_codec)
            }
        }
    }

    fn parse(&self, abi: &str) -> Box<dyn Codec> {
        let mut pairs = EthAbi::parse(Rule::Type, abi).unwrap();
        let pair = pairs.next().expect("should have a pair");
        self.accept_type(pair)
    }
}

struct Visitor;

impl Visitor {
    fn visit_zero_tuple(&self) -> Box<dyn Codec> {
        let codec = TupleCodec::new(Vec::with_capacity(0));
        Box::new(codec)
    }

    fn visit_non_zero_tuple(&self, codecs: Vec<Box<dyn Codec>>) -> Box<dyn Codec> {
        let codec = TupleCodec::new(codecs);
        Box::new(codec)
    }
}

pub fn parse(types: &[String]) -> Box<dyn Codec> {
    let mut visitor = Visitor;
    let context = EthAbiParser::new(&mut visitor);

    let codecs = types.iter().map(|t| context.parse(t)).collect::<Vec<_>>();
    let codec = TupleCodec::new(codecs);
    Box::new(codec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_empty_tuple_codec() {
        let codec = parse(&[]);
        assert_eq!(codec.encode(&Value::Tuple(vec![])), vec![]);
    }

    #[test]
    fn test_simple_tuple_codec() {
        let abi = ["bool", "uint256"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "000000000000000000000000000000000000000000000000000000000000FFFF",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
            Value::Boolean(true),
            Value::UInt(0xFFFF_u32.into()),
        ])));
    }

    #[test]
    fn test_array_nesting_tuple_codec() {
        let abi = ["bool", "uint256[]"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
            Value::Boolean(true),
            Value::Array(vec![
                Value::UInt(3_u32.into()),
                Value::UInt(4_u32.into()),
            ]),
        ])));
    }

    #[test]
    fn test_complex_tuple_codec() {
        // (uint256, (uint256, uint256[])
        let abi = ["uint256", "(uint256, uint256[])"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
            Value::UInt(1_u32.into()),
            Value::Tuple(vec![
                Value::UInt(2_u32.into()),
                Value::Array(vec![
                    Value::UInt(3_u32.into()),
                    Value::UInt(4_u32.into()),
                ]),
            ]),
        ])));
    }

    #[test]
    fn test_more_complex_tuple_codec() {
        // (uint,uint32[],bytes10,bytes)
        let abi = ["uint", "uint32[]", "bytes10", "bytes"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000123",
            "0000000000000000000000000000000000000000000000000000000000000080",
            "3132333435363738393000000000000000000000000000000000000000000000",
            "00000000000000000000000000000000000000000000000000000000000000e0",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000456",
            "0000000000000000000000000000000000000000000000000000000000000789",
            "000000000000000000000000000000000000000000000000000000000000000d",
            "48656c6c6f2c20776f726c642100000000000000000000000000000000000000",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
                Value::UInt(0x123_u32.into()),
                Value::Array(vec![
                    Value::UInt(0x456_u32.into()),
                    Value::UInt(0x789_u32.into()),
                ]),
                Value::Bytes("1234567890".as_bytes().to_vec()),
                Value::Bytes("Hello, world!".as_bytes().to_vec()),
            ]))
        );
    }

    #[test]
    fn test_issue289_encode() {
        // (uint,uint32[],bytes10,bytes)
        let abi = ["address[]", "uint256[]", "address[]", "uint256[]", "uint256[]"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);

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

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
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
            ]))
        );
    }

    #[test]
    fn test_empty_arguments() {
        let abi = vec![];
        let codec = parse(&abi);
        let value = codec.decode(&[]);
        assert_eq!(value, Value::Tuple(Vec::new()));
    }

    #[test]
    fn test_abi_parser() {
        let abi = ["uint256", "uint256[]"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);
        
        let value = codec.decode(hex::decode(concat!(
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
    fn test_issue289_decode() {
        // github issue: https://github.com/rust-ethereum/ethabi/issues/289
        let abi = ["address[]", "uint256[]", "address[]", "uint256[]", "uint256[]"].into_iter().map(Into::into).collect::<Vec<_>>();
        let codec = parse(&abi);
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

        let value = codec.decode(bytes.as_slice());
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

