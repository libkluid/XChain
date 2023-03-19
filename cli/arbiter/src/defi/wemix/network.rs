use std::rc::Rc;
use ethabi::num_traits::ToPrimitive;
use ethabi::Value;
use contracts::eth::{EthereumContract, EthereumFunction};
use rpc::jsonrpc::Response;
use rpc::channel::{OneshotChannel};
use rpc::network::{NetworkOptions, EthereumNetwork};

use crate::defi::wemix::WemixFi;
use crate::token::Token;

#[derive(Clone)]
pub struct WemixNetwork {
    pub(crate) network: Rc<EthereumNetwork>,
    pub(crate) channel: Rc<dyn OneshotChannel<Output=Response>>,
}

impl WemixNetwork {
    pub fn websocket(endpoint: &str) -> Self {
        let network = Rc::new(EthereumNetwork::new(NetworkOptions { radix: 16 }));
        let channel = Rc::new(rpc::channel::WebsocketChannel::oneshot(endpoint));

        Self {
            network,
            channel,
        }
    }

    pub fn contract(&self, address: &str) -> EthereumContract {
        EthereumContract::new(self.network.clone(), self.channel.clone(), address)
    }

    pub async fn token(&self, address: &str, multicall_aggregator: Option<&str>) -> Token {
        let address = address.to_string();

        let name = EthereumFunction::new("name", &[], &["string"]).unwrap();
        let symbol = EthereumFunction::new("symbol", &[], &["string"]).unwrap();
        let decimals = EthereumFunction::new("decimals", &[], &["uint8"]).unwrap();

        if let Some(aggregator) = multicall_aggregator {
            let multicall = self.contract(aggregator);
            let aggregate = EthereumFunction::new("aggregate", &["(address,bytes)[]"], &["uint256", "bytes[]"]).unwrap();

            let args = Value::Array(vec![
                Value::Tuple(vec![
                    Value::address(address.as_str()).unwrap(),
                    Value::Bytes(name.encode(vec![]).unwrap()),
                ]),
                Value::Tuple(vec![
                    Value::address(address.as_str()).unwrap(),
                    Value::Bytes(symbol.encode(vec![]).unwrap()),
                ]),
                Value::Tuple(vec![
                    Value::address(address.as_str()).unwrap(),
                    Value::Bytes(decimals.encode(vec![]).unwrap()),
                ]),
            ]);

            let response = multicall.invoke(&aggregate, vec![args]).await.unwrap();
            let _block = response[0].as_uint().unwrap();
            let bytes_array = response[1].as_array().unwrap();
            
            let name = name.decode(&bytes_array[0].as_bytes().unwrap()).unwrap()[0].as_string().unwrap().to_string();
            let symbol = symbol.decode(&bytes_array[1].as_bytes().unwrap()).unwrap()[0].as_string().unwrap().to_string();
            let decimals = decimals.decode(&bytes_array[2].as_bytes().unwrap()).unwrap()[0].as_uint().unwrap().to_u8().unwrap();

            Token {
                address,
                name,
                symbol,
                decimals,
            }
        } else {
            let erc20 = EthereumContract::new(self.network.clone(), self.channel.clone(), address.as_str());
            let name = erc20.invoke(&name, vec![]).await.unwrap()[0].as_string().unwrap().to_string();
            let symbol = erc20.invoke(&symbol, vec![]).await.unwrap()[0].as_string().unwrap().to_string();
            let decimals = erc20.invoke(&decimals, vec![]).await.unwrap()[0].as_uint().unwrap().to_u8().unwrap();

            Token {
                address,
                name,
                symbol,
                decimals,
            }
        }
    }

    pub async fn wemixfi(&self, router_address: &str) -> WemixFi {
        WemixFi::async_init(self, router_address).await
    }
}
