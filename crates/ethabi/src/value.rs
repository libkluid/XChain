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
    pub fn as_address(&self) -> Option<&str> {
        match self {
            Value::Address(address) => Some(address),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<&BigInt> {
        match self {
            Value::Int(int) => Some(int),
            _ => None,
        }
    }

    pub fn as_uint(&self) -> Option<&BigUint> {
        match self {
            Value::UInt(uint) => Some(uint),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_tuple(&self) -> Option<&[Value]> {
        match self {
            Value::Tuple(tuple) => Some(tuple),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<&bool> {
        match self {
            Value::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }
}
