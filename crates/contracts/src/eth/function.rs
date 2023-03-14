use ethabi::Value;
use crate::Error;
use crate::eth::signature::encode_4bytes;

pub struct EthereumFunction {
    pub name: String,
    selector: String,
    arg_codec: Box<dyn ethabi::Codec>,
    ret_codec: Box<dyn ethabi::Codec>,
}

impl EthereumFunction {
    pub fn new(name: &str, args: &[&str], returns: &[&str]) -> Result<Self, Error> {
        let signature = format!("{}({})", name, args.join(","));
        let selector = encode_4bytes(&signature);
        let arg_codec = ethabi::parse(args)?;
        let ret_codec = ethabi::parse(returns)?;

        let function = Self {
             name: name.to_string(),
             selector,
             arg_codec,
             ret_codec,
            };
        Ok(function)
    }

    pub(crate) fn encode(&self, value: Vec<Value>) -> Result<String, Error> {
        let tuple = Value::Tuple(value);
        let encoded = match self.arg_codec.encode(&tuple) {
            Ok(encoded) => hex::encode(encoded),
            Err(ethabi::Error::InvalidData) => Err(Error::InvalidData)?,
            Err(ethabi::Error::Hex(hex_error)) => Err(hex_error)?,
            Err(uncaught_error) => panic!("uncaught error: {:?}", uncaught_error),
        };

        Ok(["0x", self.selector.as_str(), encoded.as_str()].concat())
    }

    pub(crate) fn decode(&self, data: &str) -> Result<Vec<Value>, Error> {
        let bytes = hex::decode(data)?;

        let decoded = match self.ret_codec.decode(&bytes) {
            Ok(decoded) => decoded,
            Err(ethabi::Error::InvalidData) => Err(Error::InvalidData)?,
            Err(ethabi::Error::Hex(hex_error)) => Err(hex_error)?,
            Err(uncaught_error) => panic!("uncaught error: {:?}", uncaught_error),
        };

        match decoded {
            Value::Tuple(values) => Ok(values),
            _ => panic!("Tuple decoder must return a tuple"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_signature() {
        let args = &["address"];
        let returns = &["uint256"];
        let function = EthereumFunction::new("balanceOf", args, returns).unwrap();
        assert_eq!(function.name, "balanceOf");
        assert_eq!(function.selector, "70a08231");
    }

    #[test]
    fn test_encode() {
        let args = &["address"];
        let returns = &["uint256"];
        let function = EthereumFunction::new("balanceOf", args, returns).unwrap();

        let zero_address = "0000000000000000000000000000000000000000";
        let encoded = function.encode(vec![Value::Address(zero_address.to_string())]).unwrap();
        assert_eq!(
            encoded,
            "0x70a082310000000000000000000000000000000000000000000000000000000000000000"
        )
    }

    #[test]
    fn test_decode() {
        let args = &["address"];
        let returns = &["uint256"];
        let function = EthereumFunction::new("balanceOf", args, returns).unwrap();

        let one = "0000000000000000000000000000000000000000000000000000000000000001";
        let decoded = function.decode(one).unwrap();
        assert_eq!(
            decoded,
            vec![Value::UInt(1_usize.into())],
        )
    }
}
