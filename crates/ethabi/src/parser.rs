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
use crate::Error;

struct EthAbiParser<'v> {
    visitor: &'v mut Visitor,
}

impl<'v> EthAbiParser<'v> {
    fn new(visitor: &'v mut Visitor) -> Self {
        Self {
            visitor,
        }
    }

    fn accept_type(&self, pair: pest::iterators::Pair<Rule>) -> Result<Box<dyn Codec>, Error> {
        let rule = pair.as_rule();
        let inner = pair.into_inner().next()
            .expect("Rule::Type should have an inner: Rule::TupleType or Rule::BasicType");

        match inner.as_rule() {
            Rule::TupleType => self.accept_tuple_type(inner),
            Rule::BasicType => self.accept_basic_type(inner),
            _ => unreachable!("Rule::Type can not expand to {:?}", rule),
        }
    }

    fn accept_tuple_type(&self, pair: pest::iterators::Pair<Rule>) -> Result<Box<dyn Codec>, Error> {
        let rule = pair.as_rule();
        let mut inner = pair.into_inner();

        let tuple_type = inner.next()
            .expect("Rule::TupleType should have an inner: Rule::ZeroTuple or Rule::NonZeroTuple");

        let tuple_codec = match tuple_type.as_rule() {
            Rule::ZeroTuple => self.visitor.visit_zero_tuple(),
            Rule::NonZeroTuple => {
                let codecs = tuple_type.into_inner()
                    .map(|pair| self.accept_type(pair))
                    .collect::<Result<Vec<_>, Error>>()?;
                self.visitor.visit_non_zero_tuple(codecs)
            }
            _ => unreachable!("Rule::TupleType can not expand to {:?}", rule),
        };

        match inner.next() {
            None => Ok(tuple_codec),
            Some(array) => self.accept_array(array, tuple_codec),
        }
    }

    fn accept_array(&self, pair: pest::iterators::Pair<Rule>, codec: Box<dyn Codec>) -> Result<Box<dyn Codec>, Error> {
        let rule = pair.as_rule();

        let mut codec = codec;
        for pair in pair.into_inner().rev() {
            let array_codec: Box<dyn Codec> = match pair.as_rule() {
                Rule::DynamicArray => Box::new(DynamicArrayCodec::new(codec)),
                Rule::ConstArray => {
                    let digits = pair.into_inner().next()
                        .expect("Rule::ConstArray should have an inner: Rule::Digits");
                    let size = digits.as_str().parse::<usize>()
                        .expect("Rule::Digits should be a number");
                    
                    Box::new(FixedArrayCodec::new(size, codec))
                }
                _ => unreachable!("Rule::Array can not expand to {:?}", rule),
            };

            codec = array_codec;
        }
        Ok(codec)
    }

    fn accept_basic_type(&self, pair: pest::iterators::Pair<Rule>) -> Result<Box<dyn Codec>, Error> {
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

        let base_name = base.as_str().to_lowercase();
        let base_codec: Box<dyn Codec> = match base_name.as_str() {
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
                    Err(Error::UnknownType(format!("int{}", size)))?
                }
                Box::new(IntCodec::new(size))
            }
            "uint" => {
                let size = sub.map(|digits| digits.as_str().parse::<usize>().expect("Rule::Digits should be a number")).unwrap_or(256);
                if size > 256 || size % 8 != 0 {
                    Err(Error::UnknownType(format!("uint{}", size)))?
                }
                Box::new(UIntCodec::new(size))
            }
            "string" => {
                Box::new(StringCodec)
            }
            "fixed" => unimplemented!("fixed type is not supported yet"),
            "ufixed" => unimplemented!("ufixed type is not supported yet"),
            "function" => unimplemented!("function type is not supported yet"),
            _ => Err(Error::UnknownType(base_name.to_string()))?,
        };

        match array {
            None => Ok(base_codec),
            Some(array) => self.accept_array(array, base_codec)
        }
    }

    fn parse(&self, abi: &str) -> Result<Box<dyn Codec>, Error> {
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

pub fn parse(types: &[&str]) -> Result<Box<dyn Codec>, Error> {
    let mut visitor = Visitor;
    let context = EthAbiParser::new(&mut visitor);

    let codecs = types.iter().map(|t| context.parse(t)).collect::<Result<Vec<_>, Error>>()?;
    let codec = TupleCodec::new(codecs);
    Ok(Box::new(codec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_unknown_type() {
        let abi = "qbit";
        let codec = parse(&[abi]);
        assert_eq!(codec.err(), Some(Error::UnknownType(abi.to_string())));
    }

    #[test]
    fn test_simple_tuple_codec() {
        let abi = &["bool", "uint256"];
        let codec = parse(abi).unwrap();

        let bytes = hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "000000000000000000000000000000000000000000000000000000000000FFFF",
        )).unwrap();

        assert_eq!(
            bytes,
            codec.encode(&Value::Tuple(vec![
                Value::Boolean(true),
                Value::UInt(0xFFFF_u32.into()),
            ])).unwrap()
        );
    }

    #[test]
    fn test_array_nesting_tuple_codec() {
        let abi = &["bool", "uint256[]"];
        let codec = parse(abi).unwrap();

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
            ])).unwrap()
        );
    }

    #[test]
    fn test_complex_tuple_codec() {
        // (uint256, (uint256, uint256[])
        let abi = &["uint256", "(uint256, uint256[])"];
        let codec = parse(abi).unwrap();

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
            ])).unwrap()
        );
    }

    #[test]
    fn test_more_complex_tuple_codec() {
        // (uint,uint32[],bytes10,bytes)
        let abi = &["uint", "uint32[]", "bytes10", "bytes"];
        let codec = parse(abi).unwrap();

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
            ])).unwrap()
        );
    }

    #[test]
    fn test_issue289_encode() {
        // (uint,uint32[],bytes10,bytes)
        let abi = &["address[]", "uint256[]", "address[]", "uint256[]", "uint256[]"];
        let codec = parse(abi).unwrap();

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
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("2222222222222222222222222222222222222222").unwrap(),
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("2222222222222222222222222222222222222222").unwrap(),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(2_u8.into()),
                    Value::UInt(3_u8.into()),
                    Value::UInt(4_u8.into()),
                    Value::UInt(5_u8.into()),
                ]),
                Value::Array(vec![
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("2222222222222222222222222222222222222222").unwrap(),
                ]),
                Value::Array(vec![
                    Value::UInt(20_u8.into()),
                    Value::UInt(25_u8.into()),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(0_u8.into()),
                ])
            ])).unwrap()
        );
    }

    #[test]
    fn test_empty_arguments() {
        let codec = parse(&[]).unwrap();
        let value = codec.decode(&[]).unwrap();
        assert_eq!(value, Value::Tuple(Vec::new()));
    }

    #[test]
    fn test_abi_parser() {
        let abi = &["uint256", "uint256[]"];
        let codec = parse(abi).unwrap();
        
        let value = codec.decode(hex::decode(concat!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000040",
            "0000000000000000000000000000000000000000000000000000000000000002",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
        ))
        .unwrap()
        .as_slice())
        .unwrap();

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
        let abi = &["address[]", "uint256[]", "address[]", "uint256[]", "uint256[]"];
        let codec = parse(abi).unwrap();
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

        let value = codec.decode(bytes.as_slice()).unwrap();
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::Array(vec![
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("2222222222222222222222222222222222222222").unwrap(),
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("2222222222222222222222222222222222222222").unwrap(),
                ]),
                Value::Array(vec![
                    Value::UInt(1_u8.into()),
                    Value::UInt(2_u8.into()),
                    Value::UInt(3_u8.into()),
                    Value::UInt(4_u8.into()),
                    Value::UInt(5_u8.into()),
                ]),
                Value::Array(vec![
                    Value::address("1111111111111111111111111111111111111111").unwrap(),
                    Value::address("2222222222222222222222222222222222222222").unwrap(),
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

