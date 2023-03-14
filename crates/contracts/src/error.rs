#[derive(Debug, Error)]
pub enum Error {
    #[error("abi error: {0}")]
    AbiError(#[from] ethabi::Error),

    #[error("Invalid Data")]
    InvalidData,

    #[error("Hex Error")]
    HexError(#[from] hex::FromHexError),

    #[error("RPC Error")]
    RpcError(#[from] rpc::Error),
}
