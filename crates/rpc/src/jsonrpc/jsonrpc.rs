use serde::Serialize;
use crate::jsonrpc::{Id, Params};

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpc {
    pub jsonrpc: &'static str,
    pub method: &'static str,
    pub params: Params,
    pub id: Id,
}

impl JsonRpc {
    pub fn format<P>(id: Id, method: &'static str, params: P) -> Self
    where
        P: Into<Params>,
    {
        Self {
            jsonrpc: "2.0",
            method,
            params: params.into(),
            id,
        }
    }
}
