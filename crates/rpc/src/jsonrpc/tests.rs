use crate::JsonRpc;
use crate::jsonrpc::Id;

#[test]
fn test_jsonrpc() {
    let id: Id = 1_u32.into();
    let jsonrpc = JsonRpc::format(id, "test", json!(null));
    let json = serde_json::to_string(&jsonrpc).unwrap();
    assert_eq!(json, r#"{"jsonrpc":"2.0","method":"test","params":null,"id":1}"#);
}
