use std::rc::Rc;
use crate::channel;
use crate::jsonrpc;

pub struct NetworkOptions {
    pub oneshot: Rc<dyn channel::OneshotChannel<Output=jsonrpc::Response>>,
    pub subscription: Option<Rc<dyn channel::SubscriptionChannel<Item=jsonrpc::JsonRpc>>>,
    pub radix: u32,
}
