use std::cell::Cell;
use std::rc::Rc;
use num_bigint::BigInt;
use num_traits::Num;
use crate::Error;
use crate::channel;
use crate::channel::Subscriber;
use crate::jsonrpc::{self, JsonRpc};
use crate::network::NetworkOptions;

pub struct EthereumNetwork {
    sequence: Cell<u64>,
    oneshot: Rc<dyn channel::OneshotChannel<Output=jsonrpc::Response>>,
    subscription: Option<Rc<dyn channel::SubscriptionChannel<Item=jsonrpc::JsonRpc>>>,
    options: NetworkOptions,
}

impl EthereumNetwork {
    pub fn new(options: NetworkOptions) -> Self {
        Self {
            sequence: Cell::new(0),
            oneshot: options.oneshot.clone(),
            subscription: options.subscription.clone(),
            options,
        }
    }

    fn advance(&self) -> u64 {
        let next_value = self.sequence.get().wrapping_add(1);
        self.sequence.replace(next_value)
    }

    pub async fn chain_id(&self) -> Result<BigInt, Error> {
        let jsonrpc = JsonRpc::format(self.advance(), "eth_chainId", json!(null));
        expect_bigint_response(jsonrpc, self.oneshot.as_ref(), self.options.radix).await
    }

    pub async fn block_number(&self) -> Result<BigInt, Error> {
        let jsonrpc = JsonRpc::format(self.advance(), "eth_blockNumber", json!(null));
        expect_bigint_response(jsonrpc, self.oneshot.as_ref(), self.options.radix).await
    }

    pub async fn block_by_number<D>(&self, number: u64) -> Result<D, Error>
    where
        for <'de> D: serde::Deserialize<'de>
    {
        let params = json!([format!("{:#x}", number), false]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_getBlockByNumber", params);
        expect_json_response::<D>(jsonrpc, self.oneshot.as_ref()).await
    }

    pub async fn gas_price(&self) -> Result<BigInt, Error> {
        let jsonrpc = JsonRpc::format(self.advance(), "eth_gasPrice", json!(null));
        expect_bigint_response(jsonrpc, self.oneshot.as_ref(), self.options.radix).await
    }

    pub async fn code(&self, address: &str) -> Result<Vec<u8>, Error> {
        let params = json!([address,  "latest"]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_getCode", params);
        expect_bytes_response(jsonrpc, self.oneshot.as_ref()).await
    }

    pub async fn balance(&self, address: &str) -> Result<BigInt, Error> {
        let params = json!([address,  "latest"]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_getBalance", params);
        expect_bigint_response(jsonrpc, self.oneshot.as_ref(), self.options.radix).await
    }

    pub async fn transaction_count(&self, address: &str) -> Result<BigInt, Error> {
        let params = json!([address,  "latest"]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_getTransactionCount", params);
        expect_bigint_response(jsonrpc, self.oneshot.as_ref(), self.options.radix).await
    }

    pub async fn transaction_receipt<D>(&self, hash: &str) -> Result<D, Error>
    where
        for <'de> D: serde::Deserialize<'de>
    {
        let params = json!([hash]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_getTransactionReceipt", params);
        expect_json_response::<D>(jsonrpc, self.oneshot.as_ref()).await
    }

    pub async fn call(&self, to: &str, data: &str) -> Result<Vec<u8>, Error> {
        let params = json!([
            {
                "to": to,
                "data": data,
            },
            "latest",
        ]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_call", params);
        expect_bytes_response(jsonrpc, self.oneshot.as_ref()).await
    }

    pub async fn subscribe(&self, topic: &str) -> Result<Subscriber<JsonRpc>, Error> {
        let subscribe_chanel = self.subscription.as_ref().ok_or(Error::SubscriptionChannelNotProvidedError)?;
        let params = json!([topic]);
        let jsonrpc = JsonRpc::format(self.advance(), "eth_subscribe", params);
        let subscriber = subscribe_chanel.subscribe(&jsonrpc).await.unwrap();
        Ok(subscriber)
    }
}

async fn expect_bigint_response(jsonrpc: JsonRpc, channel: &dyn channel::OneshotChannel<Output=jsonrpc::Response>, radix: u32) -> Result<BigInt, Error> {
    let response = channel.fire(&jsonrpc).await?;
    let result = response.as_result::<String>()?;
    bigint_from_hex(result, radix)
}

async fn expect_bytes_response(jsonrpc: JsonRpc, channel: &dyn channel::OneshotChannel<Output=jsonrpc::Response>) -> Result<Vec<u8>, Error> {
    let response = channel.fire(&jsonrpc).await?;
    let result = response.as_result::<String>()?;
    bytes_from_hex(result)
}

async fn expect_json_response<D>(jsonrpc: JsonRpc, channel: &dyn channel::OneshotChannel<Output=jsonrpc::Response>) -> Result<D, Error>
where
    for <'de> D: serde::Deserialize<'de>
{
    let response = channel.fire(&jsonrpc).await?;
    response.as_result::<D>()
}

fn strip_hex(hex: &str) -> &str {
    match hex.starts_with("0x") {
        true => &hex[2..],
        false => hex,
    }
}

fn bigint_from_hex(hex: String, radix: u32) -> Result<BigInt, Error> {
    let stripped = strip_hex(hex.as_str());

    BigInt::from_str_radix(stripped, radix)
        .map_err(|_| Error::HexDecodeError(hex))
}

fn bytes_from_hex(hex: String) -> Result<Vec<u8>, Error> {
    let stripped = strip_hex(hex.as_str());

    hex::decode(stripped)
        .map_err(|_| Error::HexDecodeError(hex))
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;
    use crate::channel::{HttpChannel, WebsocketChannel};

    fn setup_ethereum_network() -> EthereumNetwork {
        let options = NetworkOptions {
            radix: 16,
            oneshot: Rc::new(HttpChannel::new("https://ethereum.blockpi.network/v1/rpc/public")),
            subscription: Some(Rc::new(WebsocketChannel::subscription("wss://mainnet.infura.io/ws/v3/8373ce611754454884132be22b562e45"))),
        };
        let network = EthereumNetwork::new(options);
        network
    }

    #[tokio::test]
    async fn requests_ethereum_chain_id() {
        let network = setup_ethereum_network();
        let chain_id = network.chain_id().await.unwrap();
        assert!(dbg!(chain_id) > BigInt::zero());
    }

    #[tokio::test]
    async fn requests_ethereum_block_number() {
        let network = setup_ethereum_network();
        let block_number = network.block_number().await.unwrap();
        assert!(dbg!(block_number) > BigInt::zero());
    }

    #[tokio::test]
    async fn requests_ethereum_block_by_number() {
        let network = setup_ethereum_network();
        let block = network.block_by_number::<serde_json::Value>(1).await.unwrap();
        assert!(dbg!(block).is_object());
    }

    #[tokio::test]
    async fn requests_ethereum_gas_price() {
        let network = setup_ethereum_network();
        let gas_price = network.gas_price().await.unwrap();
        assert!(dbg!(gas_price) > BigInt::zero());
    }

    #[tokio::test]
    async fn requests_ethereum_code() {
        const MULTICALL2: &'static str = "0x5BA1e12693Dc8F9c48aAD8770482f4739bEeD696";
        let network = setup_ethereum_network();
        let gas_price = network.code(MULTICALL2).await.unwrap();
        assert!(dbg!(gas_price.len()) > 0);
    }

    #[tokio::test]
    async fn requests_ethereum_balance() {
        const WETH: &'static str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        let network = setup_ethereum_network();
        let balance = network.balance(WETH).await.unwrap();
        assert!(dbg!(balance) > BigInt::zero());
    }

    #[tokio::test]
    async fn requests_ethereum_subscribe() {
        use futures::StreamExt;
        let network = setup_ethereum_network();
        let mut subscriber = network.subscribe("newHeads").await.unwrap();

        let item = subscriber.next().await;
        assert_eq!(item.is_some(), true);
        let result = item.unwrap();
        assert_eq!(result.is_ok(), true);
        let response = result.unwrap();
        assert_eq!(response.method, "eth_subscription");
    }

    #[tokio::test]
    async fn requests_ethereum_transaction_count() {
        const WETH: &'static str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        let network = setup_ethereum_network();
        let transaction_count = network.transaction_count(WETH).await.unwrap();
        assert!(dbg!(transaction_count) > BigInt::zero());
    }

    #[tokio::test]
    async fn request_ethereum_call() {
        const WETH: &'static str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        let network = setup_ethereum_network();
        let response = network.call(
            WETH,
            "0x95d89b41"
        ).await.unwrap();
        assert_eq!(dbg!(response.len()), 96);
    }
}
