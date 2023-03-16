use crate::{JsonRpc, Error};

#[async_trait]
pub trait SubscriptionChannel {
    type Item;
    async fn subscribe(&self, jsonrpc: &JsonRpc) -> Result<Subscriber<Self::Item>, Error>;
}

#[pin_project]
pub struct Subscriber<Item> {
    #[pin]
    stream: Box<dyn futures::Stream<Item=Result<Item, Error>> + Unpin>,
}


impl<Item> Subscriber<Item> {
    pub fn new(stream: Box<dyn futures::Stream<Item=Result<Item, Error>> + Unpin>) -> Self {
        Self { stream }
    }
}

impl<Item> futures::Stream for Subscriber<Item> {
    type Item = Result<Item, Error>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx)
    }
}
