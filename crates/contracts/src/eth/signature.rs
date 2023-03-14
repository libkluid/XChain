use tiny_keccak::{Hasher, Keccak};

pub fn encode_4bytes(signature: &str) -> String {
    let mut output = [0; 4];
    {
        let mut hasher = Keccak::v256();
        hasher.update(signature.as_bytes());
        hasher.finalize(&mut output);
    }
    hex::encode(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_4bytes() {
        assert_eq!(
            encode_4bytes("balanceOf(address)"),
            "70a08231",
        );
    }
}
