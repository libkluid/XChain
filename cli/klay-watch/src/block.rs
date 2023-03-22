use serde::{Deserializer, Deserialize};
use ethabi::num_traits::Num;
use ethabi::num_bigint::BigUint;

#[derive(Debug, Deserialize)]
pub struct Event<T> {
    pub result: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHead {
    #[serde(deserialize_with = "deserialize_hex")]
    pub base_fee_per_gas: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub difficulty: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub gas_limit: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub gas_used: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub nonce: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub number: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub size: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub timestamp: BigUint,

    #[serde(deserialize_with = "deserialize_hex")]
    pub total_difficulty: BigUint,

    #[serde(default)]
    pub transactions: Vec<Transaction>,

    pub extra_data: String,
    pub state_root: String,
    pub hash: String,
    pub logs_bloom: String,
    pub miner: String,
    pub mix_hash: String,
    pub parent_hash: String,
    pub receipts_root: String,
    pub sha3_uncles: String,
    pub transactions_root: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub block_hash: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub block_number: BigUint,
    pub from: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub gas: BigUint,
    #[serde(deserialize_with = "deserialize_hex")]
    pub gas_price: BigUint,
    pub hash: String,
    pub input: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub nonce: BigUint,
    pub to: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub transaction_index: BigUint,
    #[serde(deserialize_with = "deserialize_hex")]
    pub value: BigUint,
    #[serde(deserialize_with = "deserialize_hex")]
    pub v: BigUint,
    pub r: String,
    pub s: String,
}

fn deserialize_hex<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    Ok(BigUint::from_str_radix(s, 16).unwrap())
}
