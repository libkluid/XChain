use crate::{Value, Error};

pub(crate) mod sealed {
    use super::Value;
    use super::Error;

    pub trait AbiType {
        fn name(&self) -> &str;
        fn is_dynamic(&self) -> bool;
    }

    pub trait Encoder: AbiType {
        fn encode_frame(&self, value: &Value) -> Result<Vec<u8>, Error>;
    }

    pub trait Decoder: AbiType {
        fn decode_frame(&self, bytes: &[u8], offset: usize) -> Result<Value, Error>;
    }
}

pub trait Encoder: sealed::Encoder {
    fn encode(&self, value: &Value) -> Result<Vec<u8>, Error> {
        self.encode_frame(value)
    }
}

pub trait Decoder: sealed::Decoder {
    fn decode(&self, bytes: &[u8]) -> Result<Value, Error> {
        self.decode_frame(bytes, 0)
    }
}

pub trait Codec: Encoder + Decoder {}

impl<T: sealed::Decoder> Decoder for T {}
impl<T: sealed::Encoder> Encoder for T {}
impl<T: Encoder + Decoder> Codec for T {}
