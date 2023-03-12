use crate::Value;

pub(crate) mod sealed {
    use crate::Value;

    pub trait Decoder {
        fn is_dynamic(&self) -> bool;
        fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value;
    }
}

pub trait Decoder: sealed::Decoder {
    fn decode(&self, bytes: &[u8]) -> Value {
        self.decode_frame(bytes, 0)
    }
}

impl<T: sealed::Decoder> Decoder for T {}
