use crate::{JsonRpc, Error};

#[async_trait]
pub trait OneshotChannel {
    type Output;
    async fn fire(&self, jsonrpc: &JsonRpc) -> Result<Self::Output, Error>;
}
