use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tag {
    Latest,
    Earliest,
    Pending,
    #[serde(serialize_with = "serialize_block")]
    #[serde(deserialize_with = "deserialize_block")]
    Block(u64),
}

fn serialize_block<S>(block: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{:#x}", block))
}

fn deserialize_block<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    u64::from_str_radix(s, 16).map_err(serde::de::Error::custom)
}
