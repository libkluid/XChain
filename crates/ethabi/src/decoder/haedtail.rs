use num_traits::ToPrimitive;
use crate::decoder::sealed;
use crate::decoder::Decoder;
use crate::decoder::UIntDecoder;
use crate::Value;

pub struct HeadTailDecoder {
    decoder: Box<dyn Decoder>,
}

impl HeadTailDecoder {
    pub fn new(decoder: Box<dyn Decoder>) -> Self {
        Self { decoder }
    }
}

impl sealed::Decoder for HeadTailDecoder {
    fn is_dynamic(&self) -> bool { true }

    fn decode_frame(&self, bytes: &[u8], offset: usize) -> Value {
        let head = UIntDecoder::new(256).decode_frame(bytes, offset).to_uint().expect("head is uint");
        let frame_base = head.to_usize().unwrap();
        self.decoder.decode_frame(bytes, frame_base)
    }
}
