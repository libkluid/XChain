use crate::Value;

pub(crate) mod sealed {
    use crate::Value;

    pub trait AbiType {
        fn is_dynamic(&self) -> bool;
    }

    pub trait Encoder: AbiType {
        fn encode_frame(&self, value: &Value) -> Vec<u8>;
    }

    pub trait Decoder: AbiType {
        fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value;
    }
}

pub trait Encoder: sealed::Encoder {
    fn encode(&self, value: &Value) -> Vec<u8> {
        self.encode_frame(value)
    }
}

pub trait Decoder: sealed::Decoder {
    fn decode(&self, bytes: &[u8]) -> Value {
        self.decode_frame(bytes, 0)
    }
}

pub trait Codec: Encoder + Decoder {}

impl<T: sealed::Decoder> Decoder for T {}
impl<T: sealed::Encoder> Encoder for T {}
impl<T: Encoder + Decoder> Codec for T {}
