use crate::Value;

pub(crate) mod sealed {
    use crate::Value;

    pub trait Encoder {
        fn is_dynamic(&self) -> bool;
        fn encode_frame(&self, value: &Value) -> Vec<u8>;
    }
}

pub trait Encoder: sealed::Encoder {
    fn encode(&self, value: &Value) -> Vec<u8> {
        self.encode_frame(value)
    }
}

impl<T: sealed::Encoder> Encoder for T {}
