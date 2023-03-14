use std::rc::Rc;
use ethabi::Value;
use rpc::network::EthereumNetwork;
use crate::Error;
use crate::eth::EthereumFunction;

pub struct EthereumContract {
    network: Rc<EthereumNetwork>,
    address: String,
}

impl EthereumContract {
    pub fn new(network: Rc<EthereumNetwork>, address: &str) -> Self {
        Self {
            network: network,
            address: address.to_string(),
        }
    }
}

impl EthereumContract {
    pub async fn invoke(&self, function: &EthereumFunction, args: Vec<Value>) -> Result<Vec<Value>, Error> {
        let data = function.encode(args)?;
        let response = match self.network.call(self.address.as_str(), data.as_str()).await {
            Ok(response) => response,
            Err(rpc_error) => Err(Error::RpcError(rpc_error))?,
        };
        let hex_response = hex::encode(response);
        let result = function.decode(hex_response.as_str())?;
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rpc::channel::HttpChannel;
    use rpc::network::EthereumNetwork;
    use crate::eth::EthereumFunction;

    #[tokio::test]
    async fn test_contract_invoke() {
        let blockpi_channel = Rc::new(HttpChannel::new("https://ethereum.blockpi.network/v1/rpc/public"));
        let network = Rc::new(EthereumNetwork::new(blockpi_channel, Default::default()));

        const USDT: &'static str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
        let contract = EthereumContract::new(
            network,
            USDT,
        );

        let function = EthereumFunction::new(
            "name",
            &[],
            &["string"]
        ).unwrap();

        let results = contract.invoke(&function, vec![]).await.unwrap();
        let token_name = results[0].as_string().unwrap();

        assert_eq!(token_name, "Tether USD");
    }
}
