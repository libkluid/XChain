use ethabi::Value;
use contracts::eth::{EthereumContract, EthereumFunction};

use crate::Token;

use super::WemixNetwork;

pub struct WemixFi<'n> {
    wemix: &'n WemixNetwork,
    pub router: WemixRouter<'n>,
    pub factory: WemixFactory<'n>,
}

impl<'n> WemixFi<'n> {
    pub(crate) async fn async_init(
        wemix: &'n WemixNetwork,
        router_address: &str,
    ) -> WemixFi<'n> {

        let router = EthereumContract::new(wemix.network.clone(), wemix.channel.clone(), router_address);
        let router = WemixRouter::new(wemix, router);

        let factory = router.factory().await;

        Self {
            wemix,
            router,
            factory,
        }
    }
}

pub struct WemixRouter<'n> {
    wemix: &'n WemixNetwork,
    router: EthereumContract,
    factory: EthereumFunction,
}

impl<'n> WemixRouter<'n> {
    fn new(wemix: &'n WemixNetwork, router: EthereumContract) -> Self {
        let factory = EthereumFunction::new("factory", &[], &["address"]).unwrap();
        Self {
            wemix,
            router,
            factory,
        }
    }

    async fn weth(&self) -> String {
        let values = self.router.invoke(&self.factory, vec![]).await.unwrap();
        values[0].as_address().unwrap().to_string()
    }

    async fn factory(&self) -> WemixFactory<'n> {
        let values = self.router.invoke(&self.factory, vec![]).await.unwrap();
        let address = values[0].as_address().unwrap();
        let factory = EthereumContract::new(self.wemix.network.clone(), self.wemix.channel.clone(), address.as_str());

        WemixFactory::new(self.wemix, factory)
    }
}

pub struct WemixFactory<'n> {
    wemix: &'n WemixNetwork,
    factory: EthereumContract,
    get_pair: EthereumFunction,
}

impl<'n> WemixFactory<'n> {
    fn new(wemix: &'n WemixNetwork, factory: EthereumContract) -> Self {
        let get_pair = EthereumFunction::new("getPair", &["address", "address"], &["address"]).unwrap();
        Self {
            wemix,
            factory,
            get_pair,
        }
    }

    pub async fn pair(&self, token0: Token, token1: Token) -> WemixPair<'n> {
        let (token0, token1) = if token0 < token1 {
            (token0, token1)
        } else {
            (token1, token0)
        };
        
        let args = [token0.clone(), token1.clone()].into_iter().map(|t| Value::address(&t.address).unwrap()).collect::<Vec<_>>();
        let values = self.factory.invoke(&self.get_pair, args).await.unwrap();
        let address = values[0].as_address().unwrap();
        let pair = EthereumContract::new(self.wemix.network.clone(), self.wemix.channel.clone(), address.as_str());

        WemixPair::new(self.wemix, pair, token0, token1)
    }
}


pub struct WemixPair<'n> {
    wemix: &'n WemixNetwork,
    pair: EthereumContract,
    tokens: [Token; 2],
    token0: EthereumFunction,
    token1: EthereumFunction,
}

impl<'n> WemixPair<'n> {
    fn new(wemix: &'n WemixNetwork, pair: EthereumContract, token0: Token, token1: Token) -> WemixPair<'n> {
        let tokens = [token0, token1];
        let token0 = EthereumFunction::new("token0", &[], &["address"]).unwrap();
        let token1 = EthereumFunction::new("token1", &[], &["address"]).unwrap();
        
        Self {
            wemix,
            pair,
            tokens,
            token0: token0,
            token1: token1,
        }
    }

    pub fn address(&self) -> &str {
        self.pair.address.as_str()
    }

    pub fn token0(&self) -> &Token {
        &self.tokens[0]
    }

    pub fn token1(&self) -> &Token {
        &self.tokens[1]
    }
}
