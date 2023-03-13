use num_bigint::{BigInt, BigUint};
use crate::Error;

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
    pub fn as_address(&self) -> Result<&str, Error> {
        match self {
            Value::Address(address) => Ok(address),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_int(&self) -> Result<&BigInt, Error> {
        match self {
            Value::Int(int) => Ok(int),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_uint(&self) -> Result<&BigUint, Error> {
        match self {
            Value::UInt(uint) => Ok(uint),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_bytes(&self) -> Result<&[u8], Error> {
        match self {
            Value::Bytes(bytes) => Ok(bytes),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_string(&self) -> Result<&str, Error> {
        match self {
            Value::String(string) => Ok(string),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_array(&self) -> Result<&[Value], Error> {
        match self {
            Value::Array(array) => Ok(array),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_tuple(&self) -> Result<&[Value], Error> {
        match self {
            Value::Tuple(tuple) => Ok(tuple),
            _ => Err(Error::InvalidData),
        }
    }

    pub fn as_boolean(&self) -> Result<&bool, Error> {
        match self {
            Value::Boolean(boolean) => Ok(boolean),
            _ => Err(Error::InvalidData),
        }
    }
}
