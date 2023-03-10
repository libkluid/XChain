use serde::Serialize;
use serde_json::{Value, Map};

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Params {
    None,
    Array(Vec<Value>),
    Map(Map<String, Value>),
}
