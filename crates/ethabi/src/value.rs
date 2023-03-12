use num_bigint::{BigInt, BigUint};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Address(String),
    Boolean(bool),
    Int(BigInt),
    UInt(BigUint),
    Bytes(Vec<u8>),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
}

impl Value {
    pub fn to_address(&self) -> Option<String> {
        match self {
            Value::Address(address) => Some(address.clone()),
            _ => None,
        }
    }

    pub fn to_int(&self) -> Option<BigInt> {
        match self {
            Value::Int(int) => Some(int.clone()),
            _ => None,
        }
    }

    pub fn to_uint(&self) -> Option<BigUint> {
        match self {
            Value::UInt(uint) => Some(uint.clone()),
            _ => None,
        }
    }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Value::Bytes(bytes) => Some(bytes.clone()),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String(string) => Some(string.clone()),
            _ => None,
        }
    }

    pub fn to_array(&self) -> Option<Vec<Value>> {
        match self {
            Value::Array(array) => Some(array.clone()),
            _ => None,
        }
    }
}
