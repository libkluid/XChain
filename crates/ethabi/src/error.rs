
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("Input data is invalid")]
    InvalidData,

    #[error("Hex decoding error : {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Unknown type : {0}")]
    UnknownType(String),
}
