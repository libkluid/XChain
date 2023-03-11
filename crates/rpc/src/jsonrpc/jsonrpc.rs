use serde::Serialize;
use serde_json::Value;
use crate::jsonrpc::Id;

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpc {
    pub jsonrpc: &'static str,
    pub method: &'static str,
    pub params: Value,
    pub id: Id,
}

impl JsonRpc {
    pub fn format<I, P>(id: I, method: &'static str, params: P) -> Self
    where
        I: Into<Id>,
        P: Into<Value>,
    {
        Self {
            jsonrpc: "2.0",
            method,
            params: params.into(),
            id: id.into(),
        }
    }
}
