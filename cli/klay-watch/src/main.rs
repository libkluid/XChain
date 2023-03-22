extern crate futures;
extern crate hex;
extern crate serde;
extern crate serde_json;

use ethabi::num_traits::ToPrimitive;
use futures::StreamExt;
use rpc::network::{EthereumNetwork, NetworkOptions};
use rpc::channel::{WebsocketChannel, RoundRobinChannel};

mod block;

#[tokio::main]
async fn main() {
    let heartbeat = std::time::Duration::from_secs(20);
    let subscription_channel = WebsocketChannel::subscription("wss://public-en-cypress.klaytn.net/ws", heartbeat);
    let mut round_robin = RoundRobinChannel::new();
    round_robin.push_channel(WebsocketChannel::oneshot("wss://public-node-api.klaytnapi.com/v1/cypress/ws"));
    round_robin.push_channel(WebsocketChannel::oneshot("wss://public-en-cypress.klaytn.net/ws"));

    let klaytn = EthereumNetwork::new(NetworkOptions { radix: 16 });
    let mut subscriber = klaytn.subscribe(&subscription_channel, "newHeads").await.unwrap();

    let mut last_block = None;
    while let Some(payload) = subscriber.next().await.map(|x| x.unwrap()) {
        let event: block::Event<block::BlockHead> = serde_json::from_value(payload.params).unwrap();
        let head = event.result;
        let block_number = head.number.to_u64().unwrap();

        if last_block.is_none() {
            last_block = Some(block_number);
        }

        let from_block = last_block.unwrap() - 3;
        let until_block = block_number - 3;

        for block in from_block..until_block {
            let block: Option<block::BlockHead> = klaytn.block_by_number(&round_robin, block).await.unwrap();

            if block.is_none() {
                continue;
            }

            for tx in block.unwrap().transactions {
                println!("Transaction: {:#?}", tx);
            }
        }

        last_block = Some(block_number);
    }
}
