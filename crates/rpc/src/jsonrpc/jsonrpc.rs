use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::jsonrpc::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpc {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: Option<Id>,
}

impl JsonRpc {
    pub fn format<I, P>(id: I, method: &'static str, params: P) -> Self
    where
        I: Into<Id>,
        P: Into<Value>,
    {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params: params.into(),
            id: Some(id.into()),
        }
    }
}
