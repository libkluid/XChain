use tiny_keccak::{Hasher, Keccak};

pub fn encode_4bytes(signature: &str) -> [u8; 4] {
    let mut output = [0; 4];
    {
        let mut hasher = Keccak::v256();
        hasher.update(signature.as_bytes());
        hasher.finalize(&mut output);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_4bytes() {
        assert_eq!(
            encode_4bytes("balanceOf(address)"),
            [0x70, 0xa0, 0x82, 0x31],
        );
    }
}
