use crate::channel::OneshotChannel;
use crate::jsonrpc::{JsonRpc, Response};
use crate::Error;

pub struct HttpChannel {
    http: reqwest::Client,
    endpoint: String,
}

impl HttpChannel {
    pub fn new<E>(endpoint: E) -> Self
    where
        E: Into<String>,
    {
        Self {
            http: reqwest::Client::new(),
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl OneshotChannel for HttpChannel {
    type Output = Response;
    async fn fire(&self, json: &JsonRpc) -> Result<Self::Output, Error> {
        let response: Response = self.http.post(&self.endpoint)
            .json(json).send().await?
            .json().await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc::{JsonRpc, Id};

    #[tokio::test]
    async fn requests_ethereum_block_number() {
        const BLOCKPI_ETHEREUM: &'static str = "https://ethereum.blockpi.network/v1/rpc/public";

        let http = HttpChannel::new(BLOCKPI_ETHEREUM);
        let jsonrpc = JsonRpc::format(
            Id::Num(1),
            "eth_blockNumber",
            json!(null),
        );

        let result = http.fire(&jsonrpc)
            .await.expect("Failed to send request");
        assert_eq!(result.id, Id::Num(1));

        let result = result.as_result::<String>();
        assert_eq!(result.is_ok(), true);
    }
}
