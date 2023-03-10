use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    Num(u64),
    Str(String),
}

impl<U> From<U> for Id
where
    U: Into<u64>,
{
    fn from(id: U) -> Self {
        Self::Num(id.into())
    }
}
