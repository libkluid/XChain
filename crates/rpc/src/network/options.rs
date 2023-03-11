use std::default::Default;

pub struct NetworkOptions {
    pub radix: u32,
}

impl Default for NetworkOptions {
    fn default() -> Self {
        Self {
            radix: 16,
        }
    }
}
