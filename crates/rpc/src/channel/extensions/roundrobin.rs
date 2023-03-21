use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::{Condvar, Mutex};
use crate::jsonrpc::{JsonRpc, Response};
use crate::channel::OneshotChannel;

pub struct RoundRobinChannel {
    channels: Mutex<VecDeque<Arc<dyn OneshotChannel<Output=Response> + Send + Sync>>>,
    condvar: Condvar,
}

impl RoundRobinChannel {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.channels.lock().len()
    }

    pub fn push_channel<C>(&mut self, channel: C)
    where
        C: OneshotChannel<Output=Response> + Send + Sync + 'static,
    {
        let mut channels = self.channels.lock();
        channels.push_back(Arc::new(channel));
        self.condvar.notify_one();
    }

    pub fn pop_channel(&mut self) -> Option<Arc<dyn OneshotChannel<Output=Response> + Send + Sync>> {
        let mut channels = self.channels.lock();
        channels.pop_front()
    }
}

#[async_trait]
impl OneshotChannel for RoundRobinChannel {
    type Output = Response;

    async fn fire(&self, jsonrpc: &JsonRpc) -> Result<Self::Output, crate::Error> {
        let channel = {
            let mut channels = self.channels.lock();
            while channels.is_empty() {
                self.condvar.wait(&mut channels);
            }

            channels.pop_front().unwrap()
        };

        let response = channel.fire(jsonrpc).await;
        let mut channels = self.channels.lock();
        channels.push_back(channel);

        Ok(response?)
    }
}

#[cfg(test)]
mod tests {
    use super::RoundRobinChannel;
    use crate::channel::oneshot::OneshotChannel;
    use crate::channel::HttpChannel;
    use crate::jsonrpc::{JsonRpc, Response};
    use crate::Error;
    use crate::network::{EthereumNetwork, NetworkOptions};

    struct TestChannel {
        response: Response,
    }

    #[async_trait]
    impl OneshotChannel for TestChannel {
        type Output = Response;

        async fn fire(&self, _jsonrpc: &JsonRpc) -> Result<Self::Output, Error> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_roundrobin_channel() {
        let mut roundrobit = RoundRobinChannel::new();
        roundrobit.push_channel(HttpChannel::new("https://api.wemix.com"));
        roundrobit.push_channel(HttpChannel::new("https://public-node-api.klaytnapi.com/v1/cypress"));

        let network = EthereumNetwork::new(NetworkOptions { radix: 16 });

        for _ in 0..2 {
            let wemix_chain_id = network.chain_id(&roundrobit).await.unwrap();
            assert_eq!(wemix_chain_id, 1111.into());

            let klaytn_chain_id = network.chain_id(&roundrobit).await.unwrap();
            assert_eq!(klaytn_chain_id, 8217.into());
        }
    }
}
