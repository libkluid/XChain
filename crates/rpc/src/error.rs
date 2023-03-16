use tokio_tungstenite::tungstenite;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Hex decode error: {0}")]
    HexDecodeError(String),

    #[error("JsonRpc error: {0}")]
    JsonRpcError(serde_json::Value),

    #[error("Unhandled error: {0}")]
    UnahandledError(Box<dyn std::error::Error>),

    #[error("Websocket error: {0}")]
    WebsocketError(#[from] tungstenite::Error),

    #[error("Json format error: {0}")]
    JsonformatError(#[from] serde_json::Error),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Subscription error")]
    ResponseDroppedError,

    #[error("Subscription channel not provided")]
    SubscriptionChannelNotProvidedError,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        match err {
            _ => Error::UnahandledError(err.into())
        }
    }
}
