use crate::{JsonRpc, Error, jsonrpc::Response};

#[async_trait]
pub trait Channel {
    async fn send(&self, jsonrpc: &JsonRpc) -> Result<Response, Error>;
}

#[cfg(test)]
mod tests {
    use serde_json;
    use crate::jsonrpc::{JsonRpc, Id, Params};

    #[test]
    fn test_channel() {
        pub struct TestChannel;

        impl TestChannel {
            pub fn send(jsonrpc: JsonRpc) -> String {
                serde_json::to_string(&jsonrpc).unwrap()
            }
        }

        let jsonrpc = JsonRpc::format(Id::Num(1), "test", Params::None);
        let json = TestChannel::send(jsonrpc);
        assert_eq!(json, r#"{"jsonrpc":"2.0","method":"test","params":null,"id":1}"#);
    }
}

