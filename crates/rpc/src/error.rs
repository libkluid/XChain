#[derive(Debug, Error)]
pub enum Error {
    #[error("Hex decode error: {0}")]
    HexDecodeError(String),

    #[error("JsonRpc error: {0}")]
    JsonRpcError(serde_json::Value),

    #[error("Unhandled error: {0}")]
    UnahandledError(Box<dyn std::error::Error>),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        match err {
            _ => Error::UnahandledError(err.into())
        }
    }
}
