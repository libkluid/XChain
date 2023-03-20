use ethabi::num_bigint::BigUint;
use serde::{Deserialize, Serialize};

fn strip_hex(hex: &str) -> &str {
    if hex.starts_with("0x") {
        &hex[2..]
    } else {
        hex
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    pub(crate) address: String,
    pub(crate) name: String,
    pub(crate) symbol: String,
    pub(crate) decimals: u8,
}

impl Token {
    pub fn address(&self) -> &str {
        self.address.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn decimalize(&self, amount: &BigUint) -> String {
        let decimals = BigUint::from(10_usize).pow(self.decimals().into());
        let significand = amount / &decimals;
        let mantissa = amount % &decimals;

        format!("{}.{}", significand, mantissa).parse().unwrap()
    }
}

impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        let this_address = strip_hex(self.address.as_str());
        let this_bytes = hex::decode(this_address).unwrap();
        let other_address = strip_hex(other.address.as_str());
        let other_bytes = hex::decode(other_address).unwrap();

        this_bytes == other_bytes
    }
}

impl std::cmp::PartialOrd for Token {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this_address = strip_hex(self.address.as_str());
        let this_bytes = hex::decode(this_address).unwrap();
        let other_address = strip_hex(other.address.as_str());
        let other_bytes = hex::decode(other_address).unwrap();

        Some(this_bytes.cmp(&other_bytes))
    }
}
