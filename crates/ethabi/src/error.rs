
#[derive(Debug, Error)]
pub enum Error {
    #[error("Input data is invalid")]
    InvalidData,

    #[error("Hex decoding error : {0}")]
    Hex(#[from] hex::FromHexError),
}
