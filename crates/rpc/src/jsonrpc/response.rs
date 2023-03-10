use serde::{Serialize, Deserialize};
use crate::Error;
use crate::jsonrpc::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: Id,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

impl Response {
    pub fn as_result<T>(self) -> Result<T, Error>
    where
        for <'de> T: serde::Deserialize<'de>,
    {
        match (self.result, self.error) {
            (Some(result), None) => Ok(serde_json::from_value(result).unwrap()),
            (None, Some(error)) => Err(Error::JsonRpcError(error)),
            _ => panic!("Response::as_result() called on invalid response"),
        }
    }
}
